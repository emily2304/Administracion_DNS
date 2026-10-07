const CHECK_TYPES = ["tcp", "http"];

function emptyIp() {
  return { ip: "", weight: null, country: null, health_check: null };
}

function emptyHealthCheck(checkType) {
  return {
    check_type: checkType,
    port: checkType === "tcp" ? 80 : null,
    http_path: checkType === "http" ? "/" : null,
    expected_http_codes: checkType === "http" ? [200] : null,
    basic_auth_user: null,
    basic_auth_pass: null,
    timeout_sec: 5,
    retries: 3,
    interval_sec: 30,
  };
}

function aNumero(valor) {
  return valor === "" ? null : Number(valor);
}

export default function IpListEditor({ recordType, ips, onChange }) {
  function updateRow(index, cambios) {
    const nuevas = ips.map((fila, i) => (i === index ? { ...fila, ...cambios } : fila));
    onChange(nuevas);
  }

  function addRow() {
    onChange([...ips, emptyIp()]);
  }

  function removeRow(index) {
    onChange(ips.filter((_, i) => i !== index));
  }

  function toggleHealthCheck(index, activar) {
    updateRow(index, { health_check: activar ? emptyHealthCheck("tcp") : null });
  }

  function updateHealthCheck(index, cambios) {
    const fila = ips[index];
    updateRow(index, { health_check: { ...fila.health_check, ...cambios } });
  }

  return (
    <div className="ip-list-editor">
      <label>IPs</label>

      {ips.map((fila, index) => (
        <div key={index} className="ip-row">
          <input
            type="text"
            placeholder="192.168.1.1"
            value={fila.ip}
            onChange={(e) => updateRow(index, { ip: e.target.value })}
          />

          {/* Solo aparece si el tipo de registro es "weight" */}
          {recordType === "weight" && (
            <input
              type="number"
              min="1"
              placeholder="peso"
              value={fila.weight ?? ""}
              onChange={(e) => updateRow(index, { weight: aNumero(e.target.value) })}
            />
          )}

          {/* Solo aparece si el tipo de registro es "geo" */}
          {recordType === "geo" && (
            <input
              type="text"
              placeholder="país (ej. CR)"
              maxLength={2}
              value={fila.country ?? ""}
              onChange={(e) => updateRow(index, { country: e.target.value.toUpperCase() })}
            />
          )}

          {/* Se puede quitar una fila siempre que quede al menos una */}
          {ips.length > 1 && (
            <button type="button" onClick={() => removeRow(index)}>
              Quitar
            </button>
          )}

          <label className="checkbox">
            <input
              type="checkbox"
              checked={!!fila.health_check}
              onChange={(e) => toggleHealthCheck(index, e.target.checked)}
            />
            Health check
          </label>

          {fila.health_check && (
            <div className="health-check-fields">
              <select
                value={fila.health_check.check_type}
                onChange={(e) => updateHealthCheck(index, emptyHealthCheck(e.target.value))}
              >
                {CHECK_TYPES.map((t) => (
                  <option key={t} value={t}>{t}</option>
                ))}
              </select>

              {/* Puerto: obligatorio en TCP, opcional en HTTP */}
              <input
                type="number"
                min="1"
                max="65535"
                placeholder={fila.health_check.check_type === "http" ? "puerto (80)" : "puerto"}
                title="Puerto"
                value={fila.health_check.port ?? ""}
                onChange={(e) => updateHealthCheck(index, { port: aNumero(e.target.value) })}
              />

              {fila.health_check.check_type === "http" && (
                <>
                  <input
                    type="text"
                    placeholder="/path"
                    title="Path HTTP"
                    value={fila.health_check.http_path ?? ""}
                    onChange={(e) => updateHealthCheck(index, { http_path: e.target.value })}
                  />
                  <input
                    type="text"
                    placeholder="códigos esperados, ej: 200,204"
                    title="Códigos HTTP esperados"
                    value={(fila.health_check.expected_http_codes ?? []).join(",")}
                    onChange={(e) =>
                      updateHealthCheck(index, {
                        expected_http_codes: e.target.value
                          .split(",")
                          .map((c) => c.trim())
                          .filter((c) => c !== "")
                          .map(Number)
                          .filter((c) => Number.isInteger(c)),
                      })
                    }
                  />
                </>
              )}

              {/* Basic auth (opcional): el Health Checker lo usa si viene usuario */}
              <input
                type="text"
                placeholder="usuario basic auth (opcional)"
                title="Usuario basic auth"
                autoComplete="off"
                value={fila.health_check.basic_auth_user ?? ""}
                onChange={(e) => updateHealthCheck(index, { basic_auth_user: e.target.value })}
              />
              <input
                type="password"
                placeholder="contraseña basic auth"
                title="Contraseña basic auth"
                autoComplete="new-password"
                value={fila.health_check.basic_auth_pass ?? ""}
                onChange={(e) => updateHealthCheck(index, { basic_auth_pass: e.target.value })}
              />

              <input
                type="number"
                min="1"
                placeholder="timeout (seg)"
                title="Timeout (seg)"
                value={fila.health_check.timeout_sec ?? ""}
                onChange={(e) => updateHealthCheck(index, { timeout_sec: aNumero(e.target.value) })}
              />
              <input
                type="number"
                min="1"
                placeholder="reintentos"
                title="Reintentos"
                value={fila.health_check.retries ?? ""}
                onChange={(e) => updateHealthCheck(index, { retries: aNumero(e.target.value) })}
              />
              <input
                type="number"
                min="1"
                placeholder="intervalo (seg)"
                title="Intervalo entre pruebas (seg)"
                value={fila.health_check.interval_sec ?? ""}
                onChange={(e) => updateHealthCheck(index, { interval_sec: aNumero(e.target.value) })}
              />
            </div>
          )}
        </div>
      ))}

      {recordType !== "single" && (
        <button type="button" onClick={addRow}>
          + Agregar IP
        </button>
      )}
    </div>
  );
}
