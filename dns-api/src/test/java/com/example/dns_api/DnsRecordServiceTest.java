package com.example.dns_api.service;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.example.dns_api.dto.ClientLocation;
import com.example.dns_api.dto.DnsRecord;
import com.example.dns_api.dto.HealthCheckerLocation;
import com.example.dns_api.dto.RttMeasurement;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Map;
import java.util.Set;

class DnsRecordServiceTest {

    private final ObjectMapper objectMapper = new ObjectMapper();

    @Test
    void mapsClientIpToCountryRange() throws Exception {
        JsonNode ranges = objectMapper.readTree("""
                [
                  {"ip_start":"203.0.113.0","ip_end":"203.0.113.255","country":"CR"},
                  {"ip_start":"198.51.100.0","ip_end":"198.51.100.255","country":"US"}
                ]
                """);

        assertEquals("CR", DnsRecordService.findCountry(ranges, "203.0.113.42").orElseThrow());
    }

    @Test
    void mapsClientCityAndOptionalCoordinates() throws Exception {
        JsonNode ranges = objectMapper.readTree("""
                [
                  {"ip_start":"203.0.113.0","ip_end":"203.0.113.255",
                   "country":"CR","city":"San Jose","latitude":9.9,"longitude":-84.0}
                ]
                """);

        var location = DnsRecordService.findLocation(ranges, "203.0.113.42").orElseThrow();

        assertEquals("CR", location.country());
        assertEquals("San Jose", location.city());
        assertEquals(9.9, location.latitude());
        assertEquals(-84.0, location.longitude());
    }

    @Test
    void prefersTheMostSpecificMatchingRange() throws Exception {
        JsonNode ranges = objectMapper.readTree("""
                [
                  {"ip_start":"203.0.113.0","ip_end":"203.0.113.255","country":"US"},
                  {"ip_start":"203.0.113.32","ip_end":"203.0.113.63","country":"CR"}
                ]
                """);

        assertEquals("CR", DnsRecordService.findCountry(ranges, "203.0.113.42").orElseThrow());
    }

    @Test
    void ignoresInvalidRangesAndNonIpv4ClientAddresses() throws Exception {
        JsonNode ranges = objectMapper.readTree("""
                [
                  {"ip_start":"not-an-ip","ip_end":"203.0.113.255","country":"CR"}
                ]
                """);

        assertTrue(DnsRecordService.findCountry(ranges, "203.0.113.42").isEmpty());
        assertTrue(DnsRecordService.findCountry(ranges, "2001:db8::1").isEmpty());
    }

    @Test
    void estimatesClientFromMatchingCheckerCity() {
        var client = new DnsRecordService.IpToCountryLocation("CR", "san jose", null, null);
        var checkers = List.of(
                new HealthCheckerLocation("us", 37.7749, -122.4194, "US", "San Francisco"),
                new HealthCheckerLocation("cr", 9.9281, -84.0907, "CR", "San Jose"));

        assertEquals(new ClientLocation(9.9281, -84.0907),
                DnsRecordService.estimateClientLocation(client, checkers).orElseThrow());
    }

    @Test
    void estimatesCountryOnlyClientFromTheCountryCheckerCenter() {
        var client = new DnsRecordService.IpToCountryLocation("CR", null, null, null);
        var checkers = List.of(
                new HealthCheckerLocation("west", 9.0, -85.0, "cr", "Puntarenas"),
                new HealthCheckerLocation("east", 11.0, -83.0, "CR", "Limón"));

        assertEquals(new ClientLocation(10.0, -84.0),
                DnsRecordService.estimateClientLocation(client, checkers).orElseThrow());
    }

    @Test
    void prefersCityCheckerCenterBeforeCountryCenter() {
        var client = new DnsRecordService.IpToCountryLocation("CR", "San Jose", null, null);
        var checkers = List.of(
                new HealthCheckerLocation("sj-1", 9.0, -85.0, "CR", "San Jose"),
                new HealthCheckerLocation("sj-2", 11.0, -83.0, "cr", "san jose"),
                new HealthCheckerLocation("liberia", 10.0, -86.0, "CR", "Liberia"));

        assertEquals(new ClientLocation(10.0, -84.0),
                DnsRecordService.estimateClientLocation(client, checkers).orElseThrow());
    }

    @Test
    void cannotEstimateWhenNoCheckerMatchesClientCountry() {
        var client = new DnsRecordService.IpToCountryLocation("CR", null, null, null);
        var checkers = List.of(new HealthCheckerLocation("us", 37.7, -122.4, "US", "San Francisco"));

        assertTrue(DnsRecordService.estimateClientLocation(client, checkers).isEmpty());
    }

    @Test
    void mapsMeasurementsOnlyToTargetsFromTheRequestedDnsRecord() throws Exception {
        JsonNode rows = objectMapper.readTree("""
                [
                  {"checker_id":"cr","record_ip_id":"target-1","rtt_ms":12.5,"is_healthy":true},
                  {"checker_id":"us","record_ip_id":"outside-record","rtt_ms":2.0,"is_healthy":true},
                  {"checker_id":"es","record_ip_id":"target-1","rtt_ms":8.0,"is_healthy":false}
                ]
                """);

        List<RttMeasurement> measurements = DnsRecordService.mapMeasurements(
                rows,
                Map.of("target-1", "192.0.2.10"),
                Set.of("target-1"));

        assertEquals(2, measurements.size());
        assertEquals("192.0.2.10", measurements.get(0).targetIp());
        assertTrue(measurements.get(0).healthy());
        assertFalse(measurements.get(1).healthy());
    }

    @Test
    void serializesRoundTripFieldsForTheRustInterceptor() {
        DnsRecord record = new DnsRecord("round-trip", List.of(), 60);
        record.setClientCountry("CR");
        record.setClientLocation(new ClientLocation(9.9, -84.0));
        record.setHealthCheckers(List.of(
                new HealthCheckerLocation("cr", 9.9, -84.0, "CR", "San Jose")));
        record.setMeasurements(List.of(new RttMeasurement("cr", "192.0.2.10", 12.5, true)));

        JsonNode json = objectMapper.valueToTree(record);

        assertEquals("CR", json.path("client_country").asText());
        assertTrue(json.has("client_location"));
        assertEquals("cr", json.path("health_checkers").get(0).path("checker_id").asText());
        assertEquals("192.0.2.10", json.path("measurements").get(0).path("target_ip").asText());
        assertEquals(12.5, json.path("measurements").get(0).path("rtt_ms").asDouble());
    }
}
