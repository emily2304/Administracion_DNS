package com.example.dns_api.dto;

public class DnsRecordIp {
    private String ip;
    private Integer weight;
    private String country;
    private Boolean healthy;

    public DnsRecordIp() {
    }

    public DnsRecordIp(String ip, Integer weight, String country, Boolean healthy) {
        this.ip = ip;
        this.weight = weight;
        this.country = country;
        this.healthy = healthy;
    }

    public String getIp() {
        return ip;
    }

    public void setIp(String ip) {
        this.ip = ip;
    }

    public Integer getWeight() {
        return weight;
    }

    public void setWeight(Integer weight) {
        this.weight = weight;
    }

    public String getCountry() {
        return country;
    }

    public void setCountry(String country) {
        this.country = country;
    }

    public Boolean getHealthy() {
        return healthy;
    }

    public void setHealthy(Boolean healthy) {
        this.healthy = healthy;
    }

}
