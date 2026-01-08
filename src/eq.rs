use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EqBandCount {
    Three,
    Five,
}

impl Default for EqBandCount {
    fn default() -> Self {
        Self::Five
    }
}

impl EqBandCount {
    pub fn band_count(self) -> usize {
        match self {
            Self::Three => 3,
            Self::Five => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EqBand {
    pub frequency_hz: u32,
    pub gain_db: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EqModel {
    pub band_count: EqBandCount,
    pub bands: Vec<EqBand>,
}

impl EqModel {
    pub fn three_band() -> Self {
        Self {
            band_count: EqBandCount::Three,
            bands: vec![
                EqBand {
                    frequency_hz: 100,
                    gain_db: 0.0,
                },
                EqBand {
                    frequency_hz: 1000,
                    gain_db: 0.0,
                },
                EqBand {
                    frequency_hz: 10000,
                    gain_db: 0.0,
                },
            ],
        }
    }

    pub fn five_band() -> Self {
        Self {
            band_count: EqBandCount::Five,
            bands: vec![
                EqBand {
                    frequency_hz: 60,
                    gain_db: 0.0,
                },
                EqBand {
                    frequency_hz: 230,
                    gain_db: 0.0,
                },
                EqBand {
                    frequency_hz: 910,
                    gain_db: 0.0,
                },
                EqBand {
                    frequency_hz: 3600,
                    gain_db: 0.0,
                },
                EqBand {
                    frequency_hz: 14000,
                    gain_db: 0.0,
                },
            ],
        }
    }

    pub fn normalized(self) -> Self {
        if self.bands.len() == self.band_count.band_count() {
            self
        } else {
            match self.band_count {
                EqBandCount::Three => Self::three_band(),
                EqBandCount::Five => Self::five_band(),
            }
        }
    }

    pub fn clamp_gains(mut self, min_db: f32, max_db: f32) -> Self {
        for band in &mut self.bands {
            band.gain_db = band.gain_db.clamp(min_db, max_db);
        }
        self
    }
}

impl Default for EqModel {
    fn default() -> Self {
        Self::five_band()
    }
}
