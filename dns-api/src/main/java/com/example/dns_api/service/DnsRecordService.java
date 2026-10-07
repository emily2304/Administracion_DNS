package com.example.dns_api.service;

import com.example.dns_api.dto.DnsRecord;
import com.example.dns_api.dto.DnsRecordIp;
import com.example.dns_api.dto.ClientLocation;
import com.example.dns_api.dto.HealthCheckerLocation;
import com.example.dns_api.dto.RttMeasurement;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import java.io.IOException;
import java.net.URI;
import java.net.URLEncoder;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.OptionalLong;
import java.util.Set;
import java.util.UUID;

@Service
public class DnsRecordService {

    private static final Logger LOGGER = LoggerFactory.getLogger(DnsRecordService.class);

    @Value("${supabase.url}")
    private String supabaseUrl;

    @Value("${supabase.anon-key}")
    private String supabaseAnonKey;

    private final HttpClient httpClient;
    private final ObjectMapper objectMapper;

    public DnsRecordService() {
        this.httpClient = HttpClient.newHttpClient();
        this.objectMapper = new ObjectMapper();
    }

    public Optional<DnsRecord> findByDomain(String domain) {
        return findByDomain(domain, null);
    }

    public Optional<DnsRecord> findByDomain(String domain, String clientIp) {

        try {
            String encodedDomain = URLEncoder.encode(domain, StandardCharsets.UTF_8);

            // Buscar el registro DNS por host
            String recordsUrl = supabaseUrl
                    + "/rest/v1/dns_records"
                    + "?host=eq." + encodedDomain
                    + "&select=id,host,record_type";

            // Construir la solicitud HTTP para obtener el registro DNS
            HttpRequest recordsRequest = HttpRequest.newBuilder()
                    .uri(URI.create(recordsUrl))
                    .header("apikey", supabaseAnonKey)
                    .header("Authorization", "Bearer " + supabaseAnonKey)
                    .header("Accept", "application/json")
                    .GET()
                    .build();

            HttpResponse<String> recordsResponse = httpClient.send(
                    recordsRequest,
                    HttpResponse.BodyHandlers.ofString());

            if (recordsResponse.statusCode() != 200) {
                throw new RuntimeException(
                        "Error consultando dns_records. HTTP "
                                + recordsResponse.statusCode());
            }

            JsonNode records = objectMapper.readTree(recordsResponse.body());

            // No existe el dominio
            if (!records.isArray() || records.isEmpty()) {
                return Optional.empty();
            }

            JsonNode record = records.get(0);

            UUID recordId = UUID.fromString(record.get("id").asText());

            String recordType = record.get("record_type").asText();

            // Buscar las IP asociadas al registro
            String ipsUrl = supabaseUrl
                    + "/rest/v1/dns_record_ips"
                    + "?record_id=eq." + recordId
                    + "&select=id,ip,weight,country,is_healthy";

            // Construir la solicitud HTTP para obtener las IPs asociadas al registro
            HttpRequest ipsRequest = HttpRequest.newBuilder()
                    .uri(URI.create(ipsUrl))
                    .header("apikey", supabaseAnonKey)
                    .header("Authorization", "Bearer " + supabaseAnonKey)
                    .header("Accept", "application/json")
                    .GET()
                    .build();

            HttpResponse<String> ipsResponse = httpClient.send(
                    ipsRequest,
                    HttpResponse.BodyHandlers.ofString());

            if (ipsResponse.statusCode() != 200) {
                throw new RuntimeException(
                        "Error consultando dns_record_ips. HTTP "
                                + ipsResponse.statusCode());
            }

            JsonNode ips = objectMapper.readTree(ipsResponse.body());

            // Construir la lista de DnsRecordIp
            List<DnsRecordIp> dnsRecordIps = new ArrayList<>();
            Map<String, String> targetIpsById = new HashMap<>();
            Set<String> healthyTargetIds = new HashSet<>();

            for (JsonNode ipNode : ips) {

                String ip = ipNode.get("ip").asText();

                Integer weight = null;
                if (ipNode.hasNonNull("weight")) {
                    weight = ipNode.get("weight").asInt();
                }

                String country = null;
                if (ipNode.hasNonNull("country")) {
                    country = ipNode.get("country").asText();
                }

                boolean healthy = ipNode.path("is_healthy").asBoolean(false);

                if (ipNode.hasNonNull("id")) {
                    String targetId = ipNode.get("id").asText();
                    try {
                        UUID.fromString(targetId);
                        targetIpsById.put(targetId, ip);
                        if (healthy) {
                            healthyTargetIds.add(targetId);
                        }
                    } catch (IllegalArgumentException ignored) {
                        // Se ignoran ids que no se puedan usar como referencias de Supabase.
                    }
                }

                dnsRecordIps.add(
                        new DnsRecordIp(
                                ip,
                                weight,
                                country,
                                healthy));
            }

            // Supabase no tiene actualmente una columna TTL.
            DnsRecord dnsRecord = new DnsRecord(
                    recordType,
                    dnsRecordIps,
                    null);

            if (("geo".equalsIgnoreCase(recordType) || isRoundTrip(recordType))
                    && clientIp != null) {
                Optional<IpToCountryLocation> clientGeo = findLocationByIp(clientIp);
                clientGeo.ifPresent(location -> {
                    dnsRecord.setClientCountry(location.country());
                    dnsRecord.setClientCity(location.city());
                });

                if (isRoundTrip(recordType)) {
                    populateRoundTripData(dnsRecord, clientGeo.orElse(null), targetIpsById,
                            healthyTargetIds);
                }
            }

            // Devolver el registro DNS encontrado
            return Optional.of(dnsRecord);

        } catch (IOException e) {
            throw new RuntimeException(
                    "Error comunicándose con Supabase",
                    e);

        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();

            throw new RuntimeException(
                    "La consulta a Supabase fue interrumpida",
                    e);
        }
    }

    private Optional<IpToCountryLocation> findLocationByIp(String clientIp) {
        if (parseIpv4(clientIp).isEmpty()) {
            return Optional.empty();
        }
        String url = supabaseUrl + "/rest/v1/ip_to_country"
                + "?select=*";

        try {
            HttpRequest request = HttpRequest.newBuilder()
                    .uri(URI.create(url))
                    .header("apikey", supabaseAnonKey)
                    .header("Authorization", "Bearer " + supabaseAnonKey)
                    .header("Accept", "application/json")
                    .GET()
                    .build();

            HttpResponse<String> response = httpClient.send(
                    request,
                    HttpResponse.BodyHandlers.ofString());

            if (response.statusCode() != 200) {
                LOGGER.warn("No se pudo consultar ip_to_country. HTTP {}", response.statusCode());
                return Optional.empty();
            }

            JsonNode ranges = objectMapper.readTree(response.body());
            return findLocation(ranges, clientIp);
        } catch (IOException e) {
            LOGGER.warn("No se pudo obtener la ubicación del cliente desde ip_to_country", e);
            return Optional.empty();
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            LOGGER.warn("La consulta de ubicación del cliente fue interrumpida", e);
            return Optional.empty();
        }
    }

    static Optional<String> findCountry(JsonNode ranges, String clientIp) {
        return findLocation(ranges, clientIp).map(IpToCountryLocation::country);
    }

    static Optional<IpToCountryLocation> findLocation(JsonNode ranges, String clientIp) {
        OptionalLong clientAddress = parseIpv4(clientIp);
        if (clientAddress.isEmpty() || !ranges.isArray()) {
            return Optional.empty();
        }

        JsonNode bestMatch = null;
        long bestRangeSize = Long.MAX_VALUE;
        for (JsonNode range : ranges) {
            if (!range.hasNonNull("ip_start") || !range.hasNonNull("ip_end")
                    || !range.hasNonNull("country")) {
                continue;
            }

            OptionalLong start = parseIpv4(range.get("ip_start").asText());
            OptionalLong end = parseIpv4(range.get("ip_end").asText());
            if (start.isEmpty() || end.isEmpty() || start.getAsLong() > end.getAsLong()) {
                continue;
            }

            long rangeSize = end.getAsLong() - start.getAsLong();
            if (start.getAsLong() <= clientAddress.getAsLong()
                    && clientAddress.getAsLong() <= end.getAsLong()
                    && rangeSize < bestRangeSize) {
                bestMatch = range;
                bestRangeSize = rangeSize;
            }
        }

        if (bestMatch == null || bestMatch.get("country").asText().isBlank()) {
            return Optional.empty();
        }
        String city = bestMatch.path("city").isTextual()
                ? bestMatch.path("city").asText()
                : null;
        Double latitude = finiteNumber(bestMatch.get("latitude"));
        Double longitude = finiteNumber(bestMatch.get("longitude"));
        return Optional.of(new IpToCountryLocation(
                bestMatch.get("country").asText(), city, latitude, longitude));
    }

    private void populateRoundTripData(
            DnsRecord dnsRecord,
            IpToCountryLocation clientGeo,
            Map<String, String> targetIpsById,
            Set<String> healthyTargetIds) {
        if (clientGeo == null) {
            return;
        }

        List<HealthCheckerLocation> checkers = fetchHealthCheckerLocations();
        dnsRecord.setHealthCheckers(checkers);

        if (checkers.isEmpty()) {
            return;
        }

        Optional<ClientLocation> clientLocation = estimateClientLocation(clientGeo, checkers);
        if (clientLocation.isEmpty()) {
            return;
        }

        dnsRecord.setClientLocation(clientLocation.get());
        if (!targetIpsById.isEmpty()) {
            dnsRecord.setMeasurements(fetchRttMeasurements(targetIpsById, healthyTargetIds));
        }
    }

    static Optional<ClientLocation> estimateClientLocation(
            IpToCountryLocation clientGeo,
            List<HealthCheckerLocation> checkers) {
        ClientLocation explicitLocation = toClientLocation(
                clientGeo.latitude(), clientGeo.longitude());
        if (explicitLocation != null) {
            return Optional.of(explicitLocation);
        }

        List<HealthCheckerLocation> countryMatches = checkers.stream()
                .filter(checker -> samePlace(checker.country(), clientGeo.country()))
                .toList();
        if (clientGeo.city() != null && !clientGeo.city().isBlank()) {
            List<HealthCheckerLocation> cityMatches = countryMatches.stream()
                    .filter(checker -> samePlace(checker.city(), clientGeo.city()))
                    .toList();
            if (!cityMatches.isEmpty()) {
                return averageLocation(cityMatches);
            }
        }

        return averageLocation(countryMatches);
    }

    private static Optional<ClientLocation> averageLocation(
            List<HealthCheckerLocation> checkers) {
        if (checkers.isEmpty()) {
            return Optional.empty();
        }

        // ponytail: promedio simple; usar coordenadas IP-to-Country si se requiere más precisión.
        double latitude = 0;
        double longitude = 0;
        for (HealthCheckerLocation checker : checkers) {
            latitude += checker.latitude();
            longitude += checker.longitude();
        }
        return Optional.of(new ClientLocation(
                latitude / checkers.size(), longitude / checkers.size()));
    }

    private static boolean samePlace(String first, String second) {
        return first != null && second != null
                && first.trim().equalsIgnoreCase(second.trim());
    }

    private List<HealthCheckerLocation> fetchHealthCheckerLocations() {
        Optional<JsonNode> rows = fetchOptionalTable(
                "/rest/v1/health_checker_locations?select=checker_id,latitude,longitude,country,city",
                "health_checker_locations");
        if (rows.isEmpty() || !rows.get().isArray()) {
            return List.of();
        }

        List<HealthCheckerLocation> checkers = new ArrayList<>();
        for (JsonNode row : rows.get()) {
            String checkerId = optionalText(row.get("checker_id"));
            Double latitude = finiteNumber(row.get("latitude"));
            Double longitude = finiteNumber(row.get("longitude"));
            if (checkerId == null || toClientLocation(latitude, longitude) == null) {
                continue;
            }
            checkers.add(new HealthCheckerLocation(
                    checkerId,
                    latitude,
                    longitude,
                    optionalText(row.get("country")),
                    optionalText(row.get("city"))));
        }
        return checkers;
    }

    private List<RttMeasurement> fetchRttMeasurements(
            Map<String, String> targetIpsById,
            Set<String> healthyTargetIds) {
        String ids = String.join(",", targetIpsById.keySet());
        String filter = URLEncoder.encode("in.(" + ids + ")", StandardCharsets.UTF_8);
        String path = "/rest/v1/health_checker_measurements?record_ip_id=" + filter
                + "&select=checker_id,record_ip_id,rtt_ms,is_healthy";
        Optional<JsonNode> rows = fetchOptionalTable(path, "health_checker_measurements");
        if (rows.isEmpty() || !rows.get().isArray()) {
            return List.of();
        }
        return mapMeasurements(rows.get(), targetIpsById, healthyTargetIds);
    }

    static List<RttMeasurement> mapMeasurements(
            JsonNode rows,
            Map<String, String> targetIpsById,
            Set<String> healthyTargetIds) {
        if (!rows.isArray()) {
            return List.of();
        }

        List<RttMeasurement> measurements = new ArrayList<>();
        for (JsonNode row : rows) {
            String checkerId = optionalText(row.get("checker_id"));
            String recordIpId = optionalText(row.get("record_ip_id"));
            JsonNode rtt = row.get("rtt_ms");
            String targetIp = targetIpsById.get(recordIpId);
            if (checkerId == null || targetIp == null || rtt == null || !rtt.isNumber()) {
                continue;
            }

            double rttMs = rtt.asDouble();
            if (!Double.isFinite(rttMs)) {
                continue;
            }
            boolean healthy = row.path("is_healthy").asBoolean(false)
                    && healthyTargetIds.contains(recordIpId);
            measurements.add(new RttMeasurement(checkerId, targetIp, rttMs, healthy));
        }
        return measurements;
    }

    private Optional<JsonNode> fetchOptionalTable(String path, String table) {
        try {
            HttpRequest request = HttpRequest.newBuilder()
                    .uri(URI.create(supabaseUrl + path))
                    .header("apikey", supabaseAnonKey)
                    .header("Authorization", "Bearer " + supabaseAnonKey)
                    .header("Accept", "application/json")
                    .GET()
                    .build();
            HttpResponse<String> response = httpClient.send(
                    request,
                    HttpResponse.BodyHandlers.ofString());
            if (response.statusCode() != 200) {
                LOGGER.warn("No se pudo consultar {}. HTTP {}", table, response.statusCode());
                return Optional.empty();
            }
            return Optional.of(objectMapper.readTree(response.body()));
        } catch (IOException e) {
            LOGGER.warn("No se pudo consultar {} en Supabase", table, e);
            return Optional.empty();
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            LOGGER.warn("La consulta de {} fue interrumpida", table, e);
            return Optional.empty();
        }
    }

    private static Double finiteNumber(JsonNode node) {
        if (node == null || !node.isNumber()) {
            return null;
        }
        double value = node.asDouble();
        return Double.isFinite(value) ? value : null;
    }

    private static String optionalText(JsonNode node) {
        return node != null && node.isTextual() && !node.asText().isBlank()
                ? node.asText()
                : null;
    }

    private static boolean isRoundTrip(String recordType) {
        return "round-trip".equalsIgnoreCase(recordType)
                || "roundtrip".equalsIgnoreCase(recordType);
    }

    private static ClientLocation toClientLocation(Double latitude, Double longitude) {
        if (latitude == null || longitude == null || latitude < -90 || latitude > 90
                || longitude < -180 || longitude > 180) {
            return null;
        }
        return new ClientLocation(latitude, longitude);
    }

    record IpToCountryLocation(
            String country,
            String city,
            Double latitude,
            Double longitude) {
    }

    static OptionalLong parseIpv4(String address) {
        if (address == null) {
            return OptionalLong.empty();
        }

        String[] octets = address.split("\\.", -1);
        if (octets.length != 4) {
            return OptionalLong.empty();
        }

        long result = 0;
        for (String octet : octets) {
            if (octet.isEmpty() || !octet.chars().allMatch(Character::isDigit)) {
                return OptionalLong.empty();
            }
            try {
                int value = Integer.parseInt(octet);
                if (value > 255) {
                    return OptionalLong.empty();
                }
                result = (result << 8) | value;
            } catch (NumberFormatException e) {
                return OptionalLong.empty();
            }
        }
        return OptionalLong.of(result);
    }
}
