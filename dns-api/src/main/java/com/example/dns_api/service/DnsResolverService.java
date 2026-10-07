package com.example.dns_api.service;

import java.io.IOException;
import java.net.DatagramPacket;
import java.net.DatagramSocket;
import java.net.InetAddress;

import java.util.Base64;
import org.springframework.stereotype.Service;
import org.springframework.beans.factory.annotation.Value;

@Service
public class DnsResolverService {
    @Value("${external.dns.server}")
    private String dnsServer;

    @Value("${external.dns.port}")
    private int dnsPort;

    public String resolve(String dnsPacketBase64) throws IOException {

        try {
            // decode del paquete DNS base64 a bytes
            byte[] dnsPacket = Base64.getDecoder().decode(dnsPacketBase64);

            // envio del paquete decodificado al servidor DNS remoto
            byte[] dnsResponse = sendDnsQuery(dnsPacket);

            // codificacion de respuesta 
            return Base64.getEncoder().encodeToString(dnsResponse);
        }
        catch (IllegalArgumentException e) {
            throw new IOException("Invalid Base64 input", e);
        }
            
    }

    private byte[] sendDnsQuery(byte[] dnsPacket) throws IOException {

        // crear server UPD
        try (DatagramSocket datagramSocket = new DatagramSocket()) {
            // timeout de 5 segs
            datagramSocket.setSoTimeout(5000);

            // Obtener la dirección IP del sevrer DNS externo
            InetAddress address = InetAddress.getByName(dnsServer);

            // Crear el paquete UDP que contiene la consulta DNS
            DatagramPacket requestPacket = new DatagramPacket(
                    dnsPacket,
                    dnsPacket.length,
                    address,
                    dnsPort);

            // envio de consulta DNS 
            datagramSocket.send(requestPacket);
            byte[] buffer = new byte[4096];

            // paquete donde almacenara respuesta
            DatagramPacket responsePacket = new DatagramPacket(
                    buffer,
                    buffer.length);

            // espera respuesta del servidor DNS remoto
            datagramSocket.receive(responsePacket);

            // copia bytes que recibio
            byte[] dnsResponse = new byte[responsePacket.getLength()];

            System.arraycopy(
                    responsePacket.getData(),
                    responsePacket.getOffset(),
                    dnsResponse,
                    0,
                    responsePacket.getLength());

            return dnsResponse;
        }
    }
    




}