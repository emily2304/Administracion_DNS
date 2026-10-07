import { useEffect, useState } from "react";
import { getIpToCountry, deleteIpToCountry } from "../../api/geoIp";
import GeoIpForm from "./GeoIpForm";

export default function GeoIpList() {
  const [entries, setEntries] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [editando, setEditando] = useState(null);
  const [creando, setCreando] = useState(false);

  useEffect(() => {
    cargar();
  }, []);

  async function cargar() {
    setLoading(true);
    setError(null);
    try {
      setEntries(await getIpToCountry());
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  async function handleDelete(id) {
    if (!confirm("¿Borrar este rango de IPs?")) return;
    try {
      await deleteIpToCountry(id);
      cargar();
    } catch (err) {
      setError(`No se pudo borrar: ${err.message}`);
    }
  }

  if (loading) return <p>Cargando...</p>;

  if (creando || editando) {
    return (
      <GeoIpForm
        initialEntry={editando}
        onSaved={() => { setEditando(null); setCreando(false); cargar(); }}
        onCancel={() => { setEditando(null); setCreando(false); }}
      />
    );
  }

  return (
    <div>
      <button onClick={() => setCreando(true)}>+ Agregar rango</button>
      {error && <p className="error">Error: {error}</p>}
      <table className="geo-ip-list">
        <thead>
          <tr><th>IP inicial</th><th>IP final</th><th>País</th><th>Ciudad</th><th>Acciones</th></tr>
        </thead>
        <tbody>
          {entries.map((e) => (
            <tr key={e.id}>
              <td>{e.ip_start}</td>
              <td>{e.ip_end}</td>
              <td>{e.country}</td>
              <td>{e.city ?? "—"}</td>
              <td>
                <button onClick={() => setEditando(e)}>Editar</button>
                <button onClick={() => handleDelete(e.id)}>Borrar</button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}