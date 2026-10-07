//! Algoritmos puros para seleccionar targets DNS.

use rand::seq::SliceRandom;
use serde::Deserialize;
use std::net::Ipv4Addr;

use crate::api_client::IpEntry;

const EARTH_RADIUS_KM: f64 = 6_371.0;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct ClientLocation {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HealthCheckerLocation {
    #[serde(rename = "checker_id")]
    pub id: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RttMeasurement {
    #[serde(rename = "checker_id")]
    pub checker_id: String,
    #[serde(rename = "target_ip")]
    pub target_ip: Ipv4Addr,
    #[serde(rename = "rtt_ms")]
    pub rtt_ms: f64,
    pub healthy: bool,
}

/// Selecciona una IP saludable para un registro GEO.
///
/// Si hay targets asociados al país solicitado, solo se consideran esos
/// targets. Si todos están unhealthy, devuelve `None` para que el resolver
/// pueda hacer fallback al DNS público. Si el país no tiene targets o no está
/// disponible, selecciona entre todos los targets saludables.
pub fn pick_geo(targets: &[IpEntry], client_country: Option<&str>) -> Option<Ipv4Addr> {
    let healthy_targets: Vec<&IpEntry> = targets.iter().filter(|entry| entry.healthy).collect();
    if healthy_targets.is_empty() {
        return None;
    }

    let requested_country = client_country
        .map(str::trim)
        .filter(|country| !country.is_empty());

    if let Some(country) = requested_country {
        let country_targets: Vec<&IpEntry> = targets
            .iter()
            .filter(|entry| same_country(entry.country.as_deref(), country))
            .collect();

        if !country_targets.is_empty() {
            let healthy_country_targets: Vec<&IpEntry> = country_targets
                .into_iter()
                .filter(|entry| entry.healthy)
                .collect();
            return choose_ip(&healthy_country_targets);
        }
    }

    choose_ip(&healthy_targets)
}

fn same_country(target_country: Option<&str>, client_country: &str) -> bool {
    target_country
        .map(str::trim)
        .map(|country| country.eq_ignore_ascii_case(client_country))
        .unwrap_or(false)
}

fn choose_ip(targets: &[&IpEntry]) -> Option<Ipv4Addr> {
    let mut rng = rand::thread_rng();
    targets.choose(&mut rng).map(|entry| entry.ip)
}

/// Calcula la distancia entre dos ubicaciones mediante la fórmula de Haversine.
/// El resultado se expresa en kilómetros.
pub fn haversine_distance(from: ClientLocation, to: ClientLocation) -> f64 {
    let latitude_delta = (to.latitude - from.latitude).to_radians();
    let longitude_delta = (to.longitude - from.longitude).to_radians();
    let from_latitude = from.latitude.to_radians();
    let to_latitude = to.latitude.to_radians();

    let a = (latitude_delta / 2.0).sin().powi(2)
        + from_latitude.cos() * to_latitude.cos() * (longitude_delta / 2.0).sin().powi(2);

    2.0 * EARTH_RADIUS_KM * a.clamp(0.0, 1.0).sqrt().asin()
}

/// Encuentra el Health Checker geográficamente más cercano.
pub fn nearest_checker(
    client_location: ClientLocation,
    checkers: &[HealthCheckerLocation],
) -> Option<&HealthCheckerLocation> {
    let mut nearest: Option<(&HealthCheckerLocation, f64)> = None;

    for checker in checkers {
        let distance = haversine_distance(
            client_location,
            ClientLocation {
                latitude: checker.latitude,
                longitude: checker.longitude,
            },
        );

        if !distance.is_finite() {
            continue;
        }

        match nearest {
            None => nearest = Some((checker, distance)),
            Some((_, best_distance)) if distance < best_distance => {
                nearest = Some((checker, distance));
            }
            Some(_) => {}
        }
    }

    nearest.map(|(checker, _)| checker)
}

/// Selecciona el target saludable con menor RTT para un Health Checker.
pub fn lowest_rtt_target(checker_id: &str, measurements: &[RttMeasurement]) -> Option<Ipv4Addr> {
    let mut best: Option<(Ipv4Addr, f64)> = None;

    for measurement in measurements {
        if measurement.checker_id != checker_id
            || !measurement.healthy
            || !measurement.rtt_ms.is_finite()
            || measurement.rtt_ms < 0.0
        {
            continue;
        }

        match best {
            None => best = Some((measurement.target_ip, measurement.rtt_ms)),
            Some((_, best_rtt)) if measurement.rtt_ms < best_rtt => {
                best = Some((measurement.target_ip, measurement.rtt_ms));
            }
            Some(_) => {}
        }
    }

    best.map(|(target_ip, _)| target_ip)
}

/// Ejecuta el flujo completo de selección ROUND-TRIP sin hacer IO.
pub fn pick_round_trip(
    client_location: ClientLocation,
    checkers: &[HealthCheckerLocation],
    measurements: &[RttMeasurement],
) -> Option<Ipv4Addr> {
    let checker = nearest_checker(client_location, checkers)?;
    lowest_rtt_target(&checker.id, measurements)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip_entry(ip: Ipv4Addr, country: Option<&str>, healthy: bool) -> IpEntry {
        IpEntry {
            ip,
            weight: None,
            country: country.map(str::to_owned),
            healthy,
        }
    }

    fn checker(id: &str, latitude: f64, longitude: f64) -> HealthCheckerLocation {
        HealthCheckerLocation {
            id: id.to_owned(),
            latitude,
            longitude,
        }
    }

    fn measurement(
        checker_id: &str,
        target_ip: Ipv4Addr,
        rtt_ms: f64,
        healthy: bool,
    ) -> RttMeasurement {
        RttMeasurement {
            checker_id: checker_id.to_owned(),
            target_ip,
            rtt_ms,
            healthy,
        }
    }

    #[test]
    fn geo_selects_target_from_requested_country() {
        let targets = vec![
            ip_entry(Ipv4Addr::new(10, 0, 0, 1), Some("CR"), true),
            ip_entry(Ipv4Addr::new(10, 0, 0, 2), Some("US"), true),
        ];

        assert_eq!(
            pick_geo(&targets, Some("CR")),
            Some(Ipv4Addr::new(10, 0, 0, 1))
        );
    }

    #[test]
    fn geo_country_comparison_is_case_insensitive_and_trimmed() {
        let targets = vec![ip_entry(Ipv4Addr::new(10, 0, 0, 1), Some(" cr "), true)];

        assert_eq!(
            pick_geo(&targets, Some(" CR ")),
            Some(Ipv4Addr::new(10, 0, 0, 1))
        );
    }

    #[test]
    fn geo_unknown_country_uses_only_healthy_targets() {
        let healthy_ip = Ipv4Addr::new(10, 0, 0, 1);
        let targets = vec![
            ip_entry(healthy_ip, Some("CR"), true),
            ip_entry(Ipv4Addr::new(10, 0, 0, 2), Some("US"), false),
        ];

        assert_eq!(pick_geo(&targets, None), Some(healthy_ip));
    }

    #[test]
    fn geo_country_without_targets_uses_healthy_fallback() {
        let allowed = [Ipv4Addr::new(10, 0, 0, 1), Ipv4Addr::new(10, 0, 0, 2)];
        let targets = vec![
            ip_entry(allowed[0], Some("CR"), true),
            ip_entry(allowed[1], Some("US"), true),
        ];

        let selected = pick_geo(&targets, Some("MX")).expect("debe existir un fallback saludable");
        assert!(allowed.contains(&selected));
    }

    #[test]
    fn geo_does_not_select_when_country_targets_are_all_unhealthy() {
        let targets = vec![
            ip_entry(Ipv4Addr::new(10, 0, 0, 1), Some("CR"), false),
            ip_entry(Ipv4Addr::new(10, 0, 0, 2), Some("US"), true),
        ];

        assert_eq!(pick_geo(&targets, Some("CR")), None);
    }

    #[test]
    fn geo_returns_none_when_no_target_is_healthy() {
        let targets = vec![
            ip_entry(Ipv4Addr::new(10, 0, 0, 1), Some("CR"), false),
            ip_entry(Ipv4Addr::new(10, 0, 0, 2), Some("US"), false),
        ];

        assert_eq!(pick_geo(&targets, None), None);
    }

    #[test]
    fn haversine_same_point_is_zero() {
        let location = ClientLocation {
            latitude: 9.9281,
            longitude: -84.0907,
        };

        assert!(haversine_distance(location, location) < f64::EPSILON);
    }

    #[test]
    fn haversine_different_points_have_positive_distance() {
        let origin = ClientLocation {
            latitude: 0.0,
            longitude: 0.0,
        };
        let destination = ClientLocation {
            latitude: 1.0,
            longitude: 1.0,
        };

        assert!(haversine_distance(origin, destination) > 0.0);
    }

    #[test]
    fn haversine_quarter_equator_is_approximately_known_distance() {
        let origin = ClientLocation {
            latitude: 0.0,
            longitude: 0.0,
        };
        let destination = ClientLocation {
            latitude: 0.0,
            longitude: 90.0,
        };

        let distance = haversine_distance(origin, destination);
        assert!((distance - 10_007.5).abs() < 1.0);
    }

    #[test]
    fn nearest_checker_selects_the_closest_checker() {
        let checkers = vec![checker("far", 0.0, 10.0), checker("near", 0.0, 1.0)];
        let client = ClientLocation {
            latitude: 0.0,
            longitude: 0.0,
        };

        assert_eq!(
            nearest_checker(client, &checkers).map(|item| item.id.as_str()),
            Some("near")
        );
    }

    #[test]
    fn nearest_checker_returns_none_for_empty_list() {
        let client = ClientLocation {
            latitude: 0.0,
            longitude: 0.0,
        };

        assert_eq!(nearest_checker(client, &[]), None);
    }

    #[test]
    fn lowest_rtt_selects_the_smallest_valid_rtt() {
        let target = Ipv4Addr::new(10, 0, 0, 2);
        let measurements = vec![
            measurement("checker-1", Ipv4Addr::new(10, 0, 0, 1), 25.0, true),
            measurement("checker-1", target, 5.0, true),
        ];

        assert_eq!(lowest_rtt_target("checker-1", &measurements), Some(target));
    }

    #[test]
    fn lowest_rtt_ignores_unhealthy_measurements() {
        let healthy_target = Ipv4Addr::new(10, 0, 0, 2);
        let measurements = vec![
            measurement("checker-1", Ipv4Addr::new(10, 0, 0, 1), 1.0, false),
            measurement("checker-1", healthy_target, 5.0, true),
        ];

        assert_eq!(
            lowest_rtt_target("checker-1", &measurements),
            Some(healthy_target)
        );
    }

    #[test]
    fn lowest_rtt_ignores_negative_values() {
        let valid_target = Ipv4Addr::new(10, 0, 0, 4);
        let measurements = vec![
            measurement("checker-1", Ipv4Addr::new(10, 0, 0, 1), -1.0, true),
            measurement("checker-1", valid_target, 10.0, true),
        ];

        assert_eq!(
            lowest_rtt_target("checker-1", &measurements),
            Some(valid_target)
        );
    }

    #[test]
    fn lowest_rtt_ignores_nan_values() {
        let valid_target = Ipv4Addr::new(10, 0, 0, 4);
        let measurements = vec![
            measurement("checker-1", Ipv4Addr::new(10, 0, 0, 2), f64::NAN, true),
            measurement("checker-1", valid_target, 10.0, true),
        ];

        assert_eq!(
            lowest_rtt_target("checker-1", &measurements),
            Some(valid_target)
        );
    }

    #[test]
    fn lowest_rtt_ignores_infinite_values() {
        let valid_target = Ipv4Addr::new(10, 0, 0, 4);
        let measurements = vec![
            measurement("checker-1", Ipv4Addr::new(10, 0, 0, 3), f64::INFINITY, true),
            measurement("checker-1", valid_target, 10.0, true),
        ];

        assert_eq!(
            lowest_rtt_target("checker-1", &measurements),
            Some(valid_target)
        );
    }

    #[test]
    fn lowest_rtt_ignores_measurements_from_other_checkers() {
        let target = Ipv4Addr::new(10, 0, 0, 2);
        let measurements = vec![
            measurement("checker-2", Ipv4Addr::new(10, 0, 0, 1), 1.0, true),
            measurement("checker-1", target, 5.0, true),
        ];

        assert_eq!(lowest_rtt_target("checker-1", &measurements), Some(target));
    }

    #[test]
    fn lowest_rtt_returns_none_without_valid_measurements() {
        let measurements = vec![measurement(
            "checker-1",
            Ipv4Addr::new(10, 0, 0, 1),
            f64::NAN,
            true,
        )];

        assert_eq!(lowest_rtt_target("checker-1", &measurements), None);
    }

    #[test]
    fn round_trip_selects_target_from_nearest_checker() {
        let checkers = vec![checker("far", 0.0, 10.0), checker("near", 0.0, 1.0)];
        let expected = Ipv4Addr::new(10, 0, 0, 2);
        let measurements = vec![
            measurement("far", Ipv4Addr::new(10, 0, 0, 1), 1.0, true),
            measurement("near", Ipv4Addr::new(10, 0, 0, 3), 20.0, true),
            measurement("near", expected, 5.0, true),
        ];
        let client = ClientLocation {
            latitude: 0.0,
            longitude: 0.0,
        };

        assert_eq!(
            pick_round_trip(client, &checkers, &measurements),
            Some(expected)
        );
    }
}
