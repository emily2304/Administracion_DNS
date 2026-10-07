pub mod selection;
pub mod standard_query;

pub use selection::{
    haversine_distance, lowest_rtt_target, nearest_checker, pick_geo, pick_round_trip,
    ClientLocation, HealthCheckerLocation, RttMeasurement,
};
pub use standard_query::{resolve, Resolution, ResolverState};
