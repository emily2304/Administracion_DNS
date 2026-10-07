package com.example.dns_api.dto;

import com.fasterxml.jackson.annotation.JsonProperty;

public record RttMeasurement(
        @JsonProperty("checker_id") String checkerId,
        @JsonProperty("target_ip") String targetIp,
        @JsonProperty("rtt_ms") double rttMs,
        boolean healthy) {
}
