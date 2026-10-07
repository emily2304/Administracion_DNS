package com.example.dns_api.controller;

import java.util.Optional;
import java.io.IOException;

import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;

import com.example.dns_api.service.DnsRecordService;
import com.example.dns_api.service.DnsResolverService;

import com.example.dns_api.dto.DnsResolverRequest;
import com.example.dns_api.dto.DnsResolverResponse;
import com.example.dns_api.dto.DnsRecord;

@RestController
public class DnsApiController {

    private final DnsResolverService dnsResolverService;
    private final DnsRecordService dnsRecordService;

    // constructor 
    public DnsApiController(
        DnsResolverService dnsResolverService,
        DnsRecordService dnsRecordService) {

        this.dnsResolverService = dnsResolverService;
        this.dnsRecordService = dnsRecordService;
    }
    
    @GetMapping("/api/exists")
    // retorna un http del tipo DnsRecord
    public ResponseEntity<DnsRecord> getDnsInfo(
        // guarda como request param el dominio que se busca
        @RequestParam String domain,
        @RequestParam(name = "client_ip", required = false) String clientIp) {

            Optional<DnsRecord> record =
                    dnsRecordService.findByDomain(domain, clientIp);

            // retorna un http con el DnsRecord si existe, o un 404 si no existe
            return record
                    .map(r -> ResponseEntity.ok(r))
                    .orElseGet(() -> ResponseEntity.notFound().build());
    }

    @PostMapping("/api/dns_resolver")
    public ResponseEntity<?> dnsResolver(
        @RequestBody DnsResolverRequest request) {
        
            try {
                String dnsResponseBase64 = dnsResolverService.resolve(request.getData());

                return ResponseEntity.ok(
                        new DnsResolverResponse(dnsResponseBase64)
                );
            
            } catch (IllegalArgumentException e) {

                return ResponseEntity
                        .badRequest()
                        .body("Invalid Base64 input");

            } catch (IOException e) {

                return ResponseEntity
                        .status(HttpStatus.GATEWAY_TIMEOUT)
                        .body("Error while fetching extern DNS server response");
            }
    }

}
