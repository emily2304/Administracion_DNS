export function esIpValida(ip) {
  if (typeof ip !== "string") return false;
  const partes = ip.trim().split(".");
  if (partes.length !== 4) return false;
  return partes.every((p) => {
    const n = Number(p);
    return Number.isInteger(n) && n >= 0 && n <= 255 && p === String(n);
  });
}

function esPuertoValido(port) {
  return Number.isInteger(port) && port >= 1 && port <= 65535;
}

function esEnteroPositivo(n) {
  return Number.isInteger(n) && n > 0;
}

function validarHealthCheck(hc, etiqueta) {
  if (!hc) return null;

  if (hc.check_type === "tcp" && !esPuertoValido(hc.port)) {
    return `${etiqueta}: el puerto TCP debe estar entre 1 y 65535`;
  }

  if (hc.check_type === "http") {
    if (hc.port != null && !esPuertoValido(hc.port)) {
      return `${etiqueta}: el puerto HTTP debe estar entre 1 y 65535`;
    }
    if (!hc.http_path || !hc.http_path.startsWith("/")) {
      return `${etiqueta}: el path HTTP debe empezar con "/"`;
    }
    const codigos = hc.expected_http_codes ?? [];
    if (codigos.length === 0 || codigos.some((c) => c < 100 || c > 599)) {
      return `${etiqueta}: indique al menos un código HTTP válido (100-599)`;
    }
  }

  const tieneUser = !!hc.basic_auth_user;
  const tienePass = !!hc.basic_auth_pass;
  if (tieneUser !== tienePass) {
    return `${etiqueta}: basic auth necesita usuario y contraseña`;
  }

  if (!esEnteroPositivo(hc.timeout_sec)) return `${etiqueta}: el timeout debe ser mayor a 0`;
  if (!esEnteroPositivo(hc.retries)) return `${etiqueta}: los reintentos deben ser mayores a 0`;
  if (!esEnteroPositivo(hc.interval_sec)) return `${etiqueta}: el intervalo debe ser mayor a 0`;

  return null;
}

export function validarRegistro(recordType, ips) {
  if (ips.length === 0) return "Debe haber al menos una IP";

  if (recordType === "single" && ips.length !== 1) {
    return 'Un registro "single" debe tener exactamente una IP';
  }

  if ((recordType === "multi" || recordType === "weight" || recordType === "round-trip")
      && ips.length < 2) {
    return `Un registro "${recordType}" debería tener al menos dos IPs`;
  }

  const vistas = new Set();

  for (let i = 0; i < ips.length; i++) {
    const fila = ips[i];
    const etiqueta = `IP #${i + 1}`;

    if (!esIpValida(fila.ip)) {
      return `${etiqueta}: formato inválido (ej. 192.168.0.10)`;
    }

    const ip = fila.ip.trim();
    if (vistas.has(ip)) return `${etiqueta}: la IP ${ip} está repetida`;
    vistas.add(ip);

    if (recordType === "weight" && !esEnteroPositivo(fila.weight)) {
      return `${etiqueta}: el peso debe ser un entero mayor a 0`;
    }

    if (recordType === "geo" && !fila.country?.trim()) {
      return `${etiqueta}: indique el país (ej. CR)`;
    }

    const errorHc = validarHealthCheck(fila.health_check, etiqueta);
    if (errorHc) return errorHc;
  }

  return null;
}

export function ipANumero(ip) {
  return ip
    .trim()
    .split(".")
    .reduce((acc, parte) => acc * 256 + Number(parte), 0);
}