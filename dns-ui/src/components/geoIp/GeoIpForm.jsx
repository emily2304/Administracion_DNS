import { useState } from "react";
import { createIpToCountry, updateIpToCountry } from "../../api/geoIp";
import { esIpValida, ipANumero } from "../../utils/validators";

export default function GeoIpForm({ initialEntry = null, onSaved, onCancel }) {
  const esEdicion = !!initialEntry;

  const [ipStart, setIpStart] = useState(initialEntry?.ip_start ?? "");
  const [ipEnd, setIpEnd] = useState(initialEntry?.ip_end ?? "");
  const [country, setCountry] = useState(initialEntry?.country ?? "");
  const [city, setCity] = useState(initialEntry?.city ?? "");
  const [error, setError] = useState(null);
  const [saving, setSaving] = useState(false);

  async function handleSubmit(e) {
    e.preventDefault();
    if (!ipStart.trim() || !ipEnd.trim() || !country.trim()) {
      setError("IP inicial, IP final y país son obligatorios");
      return;
    }
    if (!esIpValida(ipStart) || !esIpValida(ipEnd)) {
      setError("Las IPs deben tener formato válido (ej. 192.168.0.0)");
      return;
    }
    if (ipANumero(ipStart) > ipANumero(ipEnd)) {
      setError("La IP inicial debe ser menor o igual que la IP final");
      return;
    }

    // Se limpian espacios y el país va en mayúsculas (CR, US...)
    const datos = {
      ipStart: ipStart.trim(),
      ipEnd: ipEnd.trim(),
      country: country.trim().toUpperCase(),
      city: city.trim(),
    };

    setSaving(true);
    setError(null);
    try {
      if (esEdicion) {
        await updateIpToCountry(initialEntry.id, datos);
      } else {
        await createIpToCountry(datos);
      }
      onSaved?.();
    } catch (err) {
      setError(err.message);
    } finally {
      setSaving(false);
    }
  }

  return (
    <form onSubmit={handleSubmit} className="geo-ip-form">
      <div className="field">
        <label>IP inicial</label>
        <input value={ipStart} onChange={(e) => setIpStart(e.target.value)} placeholder="192.168.0.0" />
      </div>
      <div className="field">
        <label>IP final</label>
        <input value={ipEnd} onChange={(e) => setIpEnd(e.target.value)} placeholder="192.168.255.255" />
      </div>
      <div className="field">
        <label>País</label>
        <input value={country} onChange={(e) => setCountry(e.target.value)} placeholder="CR" />
      </div>
      <div className="field">
        <label>Ciudad (opcional)</label>
        <input value={city} onChange={(e) => setCity(e.target.value)} placeholder="San José" />
      </div>

      {error && <span className="error">{error}</span>}

      <div className="actions">
        <button type="submit" disabled={saving}>
          {saving ? "Guardando..." : esEdicion ? "Actualizar" : "Guardar"}
        </button>
        {onCancel && <button type="button" onClick={onCancel}>Cancelar</button>}
      </div>
    </form>
  );
}