# ---- Etapa de build ----
FROM eclipse-temurin:21-jdk AS builder

WORKDIR /app

# Copiamos los archivos necesarios para usar Maven Wrapper
COPY pom.xml mvnw ./
COPY .mvn .mvn

# Damos permisos de ejecución al Maven Wrapper
RUN chmod +x mvnw

# Descargamos las dependencias
RUN ./mvnw dependency:go-offline

# Copiamos el código fuente
COPY src ./src

# Compilamos la aplicación
RUN ./mvnw clean package -DskipTests

# ---- Etapa final ----
FROM eclipse-temurin:21-jre

WORKDIR /app

# Copiamos el JAR generado en la etapa anterior
COPY --from=builder /app/target/*.jar app.jar

# El DNS API escucha en el puerto 8080
EXPOSE 8080

# Ejecutamos la aplicación
ENTRYPOINT ["java", "-jar", "app.jar"]