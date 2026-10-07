package com.example.dns_api;

// https://www.youtube.com/watch?v=YqlxKOY2QmY
// https://stackoverflow.com/questions/41935207/base64-string-to-byte-in-java
// https://medium.com/@n3d/java-networking-basics-networking-with-udp-sockets-9c5a8e8249fc
// https://docs.oracle.com/javase/8/docs/api/java/net/DatagramPacket.html

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;

@SpringBootApplication
public class DnsApiApplication {

	public static void main(String[] args) {
		SpringApplication.run(DnsApiApplication.class, args);
	}

}
