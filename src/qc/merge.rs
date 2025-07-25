use crate::prelude::{Constellation, Header, SP3};

use qc_traits::{Merge, MergeError};

#[cfg(doc)]
use qc_traits::Timeshift;

#[cfg(doc)]
use crate::prelude::TimeScale;

impl Merge for Header {
    /// Merge `rhs` [SP3] into self, creating a new combined [SP3].
    /// For this operation to work:
    /// - data must be published by the same provider
    /// - both files must be expressed in the same [TimeScale].
    /// Use our [Timeshift] transpositions if you need it.
    /// - both files must use the same coordinates system
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError>
    where
        Self: Sized,
    {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }

    /// [SP3] mutable merge. See [Self::merge] for more information.
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        // Verifications
        if self.agency != rhs.agency {
            return Err(MergeError::DataProviderMismatch);
        }

        if self.timescale != rhs.timescale {
            return Err(MergeError::TimescaleMismatch);
        }

        if self.coord_system != rhs.coord_system {
            return Err(MergeError::ReferenceFrameMismatch);
        }

        // "upgrade" constellation
        if self.constellation != rhs.constellation {
            self.constellation = Constellation::Mixed;
        }

        // update revision
        self.version = std::cmp::min(self.version, rhs.version);

        // update time reference
        if rhs.mjd < self.mjd {
            self.mjd = rhs.mjd;
        }

        if rhs.week < self.week {
            self.week = rhs.week;
            self.week_nanos = rhs.week_nanos;
        }

        // update SV table
        for satellite in rhs.satellites.iter() {
            if !self.satellites.contains(&satellite) {
                self.satellites.push(*satellite);
            }
        }

        // update sampling
        self.sampling_period = std::cmp::max(self.sampling_period, rhs.sampling_period);

        Ok(())
    }
}

impl Merge for SP3 {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut s = self.clone();
        s.merge_mut(rhs)?;
        Ok(s)
    }
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        self.header.merge_mut(&rhs.header)?;

        for (key, entry) in &rhs.data {
            if let Some(lhs_entry) = self.data.get_mut(key) {
                if let Some(clock_us) = entry.clock_us {
                    lhs_entry.clock_us = Some(clock_us);
                }

                if let Some(drift_ns) = entry.clock_drift_ns {
                    lhs_entry.clock_drift_ns = Some(drift_ns);
                }

                if let Some((vel_x_km_s, vel_y_km_s, vel_z_km_s)) = entry.velocity_km_s {
                    lhs_entry.velocity_km_s = Some((vel_x_km_s, vel_y_km_s, vel_z_km_s));
                }
            } else {
                self.data.insert(key.clone(), entry.clone()); // new entry
            }
        }
        Ok(())
    }
}
