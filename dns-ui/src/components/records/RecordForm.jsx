import { useState } from "react";
import IpListEditor from "./IpListEditor";
import { createRecord, updateRecord } from "../../api/records";
import { validarRegistro } from "../../utils/validators";

const TIPOS = ["single", "multi", "weight", "round-trip", "geo"];

function extraerHealthCheck(config) {
  const hc = Array.isArray(config) ? config[0] : config;
  return hc ?? null;
}

function ipsDesdeRegistro(record) {
  return record.dns_record_ips.map((ip) => ({
    id: ip.id,
    ip: ip.ip,
    weight: ip.weight,
    country: ip.country,
    health_check: extraerHealthCheck(ip.health_check_configs),
  }));
}

function normalizarHost(host) {
  return host.trim().toLowerCase().replace(/\.$/, "");
}

export default function RecordForm({ initialRecord = null, onSaved, onCancel }) {
  const esEdicion = !!initialRecord;

  const [host, setHost] = useState(initialRecord?.host ?? "");
  const [recordType, setRecordType] = useState(initialRecord?.record_type ?? "single");
  const [ips, setIps] = useState(
    initialRecord
      ? ipsDesdeRegistro(initialRecord)
      : [{ ip: "", weight: null, country: null, health_check: null }]
  );
  const [errors, setErrors] = useState({});
  const [saving, setSaving] = useState(false);

  function validate() {
    const nuevosErrores = {};
    if (!normalizarHost(host)) nuevosErrores.host = "El host es obligatorio";

    const errorIps = validarRegistro(recordType, ips);
    if (errorIps) nuevosErrores.ips = errorIps;

    setErrors(nuevosErrores);
    return Object.keys(nuevosErrores).length === 0;
  }

  async function handleSubmit(e) {
    e.preventDefault();
    if (!validate()) return;

    setSaving(true);
    try {
      const datos = { host: normalizarHost(host), recordType, ips };
      const id = esEdicion
        ? await updateRecord(initialRecord.id, datos)
        : await createRecord(datos);

      // Si es un registro nuevo, se limpia el formulario para crear otro
      if (!esEdicion) {
        setHost("");
        setRecordType("single");
        setIps([{ ip: "", weight: null, country: null, health_check: null }]);
      }
      setErrors({});
      onSaved?.(id);
    } catch (err) {
      setErrors({ general: err.message });
    } finally {
      setSaving(false);
    }
  }

  return (
    <form onSubmit={handleSubmit} className="record-form">
      <div className="field">
        <label htmlFor="host">Host</label>
        <input id="host" value={host} onChange={(e) => setHost(e.target.value)} placeholder="ejemplo.com" />
        {errors.host && <span className="error">{errors.host}</span>}
      </div>

      <div className="field">
        <label htmlFor="recordType">Tipo de registro</label>
        <select id="recordType" value={recordType} onChange={(e) => setRecordType(e.target.value)}>
          {TIPOS.map((t) => <option key={t} value={t}>{t}</option>)}
        </select>
      </div>

      <IpListEditor recordType={recordType} ips={ips} onChange={setIps} />
      {errors.ips && <span className="error">{errors.ips}</span>}

      {errors.general && <span className="error">{errors.general}</span>}

      <div className="actions">
        <button type="submit" disabled={saving}>
          {saving ? "Guardando..." : esEdicion ? "Actualizar" : "Guardar"}
        </button>
        {onCancel && <button type="button" onClick={onCancel}>Cancelar</button>}
      </div>
    </form>
  );
}
