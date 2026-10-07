package com.example.dns_api.dto;

import com.fasterxml.jackson.annotation.JsonProperty;

public record HealthCheckerLocation(
        @JsonProperty("checker_id") String id,
        double latitude,
        double longitude,
        String country,
        String city) {
}
