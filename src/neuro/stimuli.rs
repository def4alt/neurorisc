use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::neuro::network::Network;
use crate::neuro::neuron::NeuronId;

#[derive(Clone, Serialize, Deserialize)]
pub struct StimulusSpec {
    pub mode: StimulusMode,
    pub enabled: bool,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum StimulusMode {
    ManualPulse {
        amplitude: f64,
    },

    Poisson {
        rate: f64,
        seed: u64,
        start: u32,
        stop: Option<u32>,
    },

    SpikeTrain {
        times: Vec<u32>,
        looped: bool,
    },

    CurrentStep {
        amp: f64,
        start: u32,
        stop: u32,
        rate: f64,
    },
}

pub struct StimulusRunner {
    dt: f64,
    stimuli: Vec<ActiveStimulus>,
}

impl StimulusRunner {
    pub fn new(dt: f64) -> Self {
        Self {
            dt,
            stimuli: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.stimuli.clear();
    }

    pub fn fire(
        &mut self,
        stimulus_id: u64,
        neuron_id: NeuronId,
        spec: &StimulusSpec,
        network: &Network,
    ) {
        let to_ticks = |ms: u32| ((ms as f64 / self.dt).max(0.0)).round() as u64;
        let base_tick = network.t as u64;
        let base_time_ms = base_tick as f64 * self.dt;

        self.stimuli
            .retain(|stimulus| stimulus.stimulus_id != stimulus_id);

        match &spec.mode {
            StimulusMode::ManualPulse { amplitude } => {
                self.stimuli.push(ActiveStimulus {
                    stimulus_id,
                    neuron_id,
                    mode: ActiveStimulusMode::ManualPulse {
                        tick: base_tick,
                        amp: *amplitude,
                    },
                });
            }
            StimulusMode::Poisson {
                rate,
                seed,
                start,
                stop,
            } => {
                if *rate <= 0.0 {
                    return;
                }

                let start_time_ms = base_time_ms + *start as f64;
                let stop_time_ms = stop.map(|s| base_time_ms + s as f64);
                if let Some(stop_ms) = stop_time_ms {
                    if stop_ms < start_time_ms {
                        return;
                    }
                }

                let mut rng = StdRng::seed_from_u64(*seed);
                let next_time_ms = start_time_ms + Self::poisson_interval_ms(*rate, &mut rng);

                self.stimuli.push(ActiveStimulus {
                    stimulus_id,
                    neuron_id,
                    mode: ActiveStimulusMode::Poisson {
                        rate: *rate,
                        rng,
                        next_time_ms,
                        stop_time_ms,
                        amp: 1.0,
                    },
                });
            }
            StimulusMode::SpikeTrain { times, looped } => {
                if times.is_empty() {
                    return;
                }

                let times_ticks: Vec<u64> = times.iter().map(|&ms| to_ticks(ms)).collect();
                let period_ticks = times_ticks.iter().copied().max().unwrap_or(0);

                self.stimuli.push(ActiveStimulus {
                    stimulus_id,
                    neuron_id,
                    mode: ActiveStimulusMode::SpikeTrain {
                        times_ticks,
                        looped: *looped && period_ticks > 0,
                        base_tick,
                        index: 0,
                        period_ticks,
                        amp: 1.0,
                    },
                });
            }
            StimulusMode::CurrentStep {
                amp,
                start,
                stop,
                rate,
            } => {
                let start_tick = base_tick.saturating_add(to_ticks(*start));
                let mut stop_tick = base_tick.saturating_add(to_ticks(*stop));
                if stop_tick < start_tick {
                    stop_tick = start_tick;
                }
                let interval_ticks = if *rate <= 0.0 {
                    1
                } else {
                    ((1000.0 / rate.max(1e-6)) / self.dt)
                        .max(1.0)
                        .round() as u64
                };

                self.stimuli.push(ActiveStimulus {
                    stimulus_id,
                    neuron_id,
                    mode: ActiveStimulusMode::CurrentStep {
                        amp: *amp,
                        start_tick,
                        stop_tick,
                        next_tick: start_tick,
                        interval_ticks,
                    },
                });
            }
        }
    }

    pub fn apply(&mut self, network: &mut Network) {
        if self.stimuli.is_empty() {
            return;
        }

        let current_tick = network.t as u64;
        let current_time_ms = current_tick as f64 * self.dt;

        let mut stimuli = std::mem::take(&mut self.stimuli);
        let mut i = 0;
        while i < stimuli.len() {
            let done = {
                let stimulus = &mut stimuli[i];
                match &mut stimulus.mode {
                    ActiveStimulusMode::ManualPulse { tick, amp } => {
                        if current_tick >= *tick {
                            network.schedule_spike(stimulus.neuron_id, *amp, 0);
                            true
                        } else {
                            false
                        }
                    }
                    ActiveStimulusMode::Poisson {
                        rate,
                        rng,
                        next_time_ms,
                        stop_time_ms,
                        amp,
                    } => {
                        if *rate <= 0.0 {
                            true
                        } else {
                            let mut events = 0;
                            loop {
                                if let Some(stop_ms) = *stop_time_ms {
                                    if *next_time_ms > stop_ms {
                                        break;
                                    }
                                }
                                if current_time_ms < *next_time_ms {
                                    break;
                                }
                                network.schedule_spike(stimulus.neuron_id, *amp, 0);
                                *next_time_ms += Self::poisson_interval_ms(*rate, rng);
                                events += 1;
                                if events >= 1024 {
                                    break;
                                }
                            }

                            if let Some(stop_ms) = *stop_time_ms {
                                current_time_ms >= stop_ms && *next_time_ms > stop_ms
                            } else {
                                false
                            }
                        }
                    }
                    ActiveStimulusMode::SpikeTrain {
                        times_ticks,
                        looped,
                        base_tick,
                        index,
                        period_ticks,
                        amp,
                    } => {
                        if times_ticks.is_empty() {
                            true
                        } else {
                            let mut done = false;
                            if *index >= times_ticks.len() {
                                if *looped && *period_ticks > 0 {
                                    *index = 0;
                                    *base_tick = base_tick.saturating_add(*period_ticks);
                                } else {
                                    done = true;
                                }
                            }

                            if !done {
                                let mut events = 0;
                                loop {
                                    let next_tick = base_tick.saturating_add(
                                        times_ticks.get(*index).copied().unwrap_or(0),
                                    );
                                    if current_tick < next_tick {
                                        break;
                                    }

                                    network.schedule_spike(stimulus.neuron_id, *amp, 0);

                                    *index += 1;
                                    if *index >= times_ticks.len() {
                                        if !*looped || *period_ticks == 0 {
                                            break;
                                        }
                                        *index = 0;
                                        *base_tick = base_tick.saturating_add(*period_ticks);
                                    }

                                    events += 1;
                                    if events >= 1024 {
                                        break;
                                    }
                                }

                                done =
                                    *index >= times_ticks.len() && (!*looped || *period_ticks == 0);
                            }

                            done
                        }
                    }
                    ActiveStimulusMode::CurrentStep {
                        amp,
                        start_tick,
                        stop_tick,
                        next_tick,
                        interval_ticks,
                    } => {
                        let mut events = 0;
                        while current_tick >= *next_tick && *next_tick <= *stop_tick {
                            if current_tick >= *start_tick {
                                network.schedule_spike(stimulus.neuron_id, *amp, 0);
                            }
                            *next_tick = next_tick.saturating_add(*interval_ticks);
                            events += 1;
                            if events >= 1024 {
                                break;
                            }
                        }

                        current_tick > *stop_tick
                    }
                }
            };

            if done {
                stimuli.remove(i);
            } else {
                i += 1;
            }
        }

        self.stimuli = stimuli;
    }

    fn poisson_interval_ms(rate_hz: f64, rng: &mut StdRng) -> f64 {
        let u: f64 = rng.random();
        let u = u.max(f64::MIN_POSITIVE);
        -u.ln() * 1000.0 / rate_hz.max(1e-6)
    }
}

struct ActiveStimulus {
    stimulus_id: u64,
    neuron_id: NeuronId,
    mode: ActiveStimulusMode,
}

enum ActiveStimulusMode {
    ManualPulse {
        tick: u64,
        amp: f64,
    },
    Poisson {
        rate: f64,
        rng: StdRng,
        next_time_ms: f64,
        stop_time_ms: Option<f64>,
        amp: f64,
    },
    SpikeTrain {
        times_ticks: Vec<u64>,
        looped: bool,
        base_tick: u64,
        index: usize,
        period_ticks: u64,
        amp: f64,
    },
    CurrentStep {
        amp: f64,
        start_tick: u64,
        stop_tick: u64,
        next_tick: u64,
        interval_ticks: u64,
    },
}
