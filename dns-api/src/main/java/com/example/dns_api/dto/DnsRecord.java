package com.example.dns_api.dto;

import com.fasterxml.jackson.annotation.JsonProperty;

import java.util.ArrayList;
import java.util.List;

public class DnsRecord {

    @JsonProperty("record_type")
    private String recordType;

    private List<DnsRecordIp> ips;
    private Integer ttl;
    @JsonProperty("client_country")
    private String clientCountry;
    @JsonProperty("client_city")
    private String clientCity;
    @JsonProperty("client_location")
    private ClientLocation clientLocation;
    @JsonProperty("health_checkers")
    private List<HealthCheckerLocation> healthCheckers = new ArrayList<>();
    private List<RttMeasurement> measurements = new ArrayList<>();

    public DnsRecord() {
    }

    public DnsRecord(String recordType, List<DnsRecordIp> ips, Integer ttl) {
        this.recordType = recordType;
        this.ips = ips;
        this.ttl = ttl;
    }

    public String getRecordType() {
        return recordType;
    }

    public void setRecordType(String recordType) {
        this.recordType = recordType;
    }

    public List<DnsRecordIp> getIps() {
        return ips;
    }

    public void setIps(List<DnsRecordIp> ips) {
        this.ips = ips;
    }

    public Integer getTtl() {
        return ttl;
    }

    public void setTtl(Integer ttl) {
        this.ttl = ttl;
    }

    public String getClientCountry() {
        return clientCountry;
    }

    public void setClientCountry(String clientCountry) {
        this.clientCountry = clientCountry;
    }

    public String getClientCity() {
        return clientCity;
    }

    public void setClientCity(String clientCity) {
        this.clientCity = clientCity;
    }

    public ClientLocation getClientLocation() {
        return clientLocation;
    }

    public void setClientLocation(ClientLocation clientLocation) {
        this.clientLocation = clientLocation;
    }

    public List<HealthCheckerLocation> getHealthCheckers() {
        return healthCheckers;
    }

    public void setHealthCheckers(List<HealthCheckerLocation> healthCheckers) {
        this.healthCheckers = healthCheckers;
    }

    public List<RttMeasurement> getMeasurements() {
        return measurements;
    }

    public void setMeasurements(List<RttMeasurement> measurements) {
        this.measurements = measurements;
    }
}
