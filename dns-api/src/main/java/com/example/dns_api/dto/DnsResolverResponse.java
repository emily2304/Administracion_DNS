package com.example.dns_api.dto;

public class DnsResolverResponse {
    private String data;

    public DnsResolverResponse(String data) {
        this.data = data;
    }

    public String getData() {
        return data;
    }

    public void setData(String data) {
        this.data = data;
    }
    
}
