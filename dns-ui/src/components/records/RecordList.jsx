import { useEffect, useState } from "react";
import { getRecords, deleteRecord } from "../../api/records";
import RecordForm from "./RecordForm";

function EstadoIp({ ip }) {
  if (ip.is_healthy === true) {
    const rtt = ip.rtt_ms != null ? ` · ${Number(ip.rtt_ms).toFixed(1)} ms` : "";
    return <span title="healthy">✅{rtt}</span>;
  }
  if (ip.is_healthy === false) return <span title="unhealthy">❌ unhealthy</span>;
  return <span title="Sin health check o aún no revisada">⏳ sin revisar</span>;
}

export default function RecordList() {
  const [records, setRecords] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [editando, setEditando] = useState(null); // registro que se está editando, o null

  useEffect(() => {
    cargarRegistros();
  }, []);

  async function cargarRegistros() {
    setLoading(true);
    setError(null);
    try {
      setRecords(await getRecords());
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  async function handleDelete(id) {
    if (!confirm("¿Borrar este registro? Esta acción no se puede deshacer.")) return;
    try {
      await deleteRecord(id);
      cargarRegistros();
    } catch (err) {
      setError(`No se pudo borrar: ${err.message}`);
    }
  }

  if (loading) return <p>Cargando registros...</p>;

  if (editando) {
    return (
      <RecordForm
        initialRecord={editando}
        onSaved={() => { setEditando(null); cargarRegistros(); }}
        onCancel={() => setEditando(null)}
      />
    );
  }

  return (
    <div>
      <div className="actions">
        <button type="button" onClick={cargarRegistros}>↻ Actualizar estado</button>
      </div>

      {error && <p className="error">Error: {error}</p>}

      {records.length === 0 ? (
        <p>No hay registros todavía.</p>
      ) : (
        <table className="record-list">
          <thead>
            <tr><th>Host</th><th>Tipo</th><th>IPs</th><th>Estado</th><th>Acciones</th></tr>
          </thead>
          <tbody>
            {records.map((r) => (
              <tr key={r.id}>
                <td>{r.host}</td>
                <td>{r.record_type}</td>
                <td>
                  {r.dns_record_ips.map((ip) => (
                    <div key={ip.id}>
                      {ip.ip}
                      {ip.weight != null && ` (peso: ${ip.weight})`}
                      {ip.country && ` (${ip.country})`}
                    </div>
                  ))}
                </td>
                <td>
                  {/* Una línea por IP, en el mismo orden que la columna IPs */}
                  {r.dns_record_ips.map((ip) => (
                    <div key={ip.id}><EstadoIp ip={ip} /></div>
                  ))}
                </td>
                <td>
                  <button onClick={() => setEditando(r)}>Editar</button>
                  <button onClick={() => handleDelete(r.id)}>Borrar</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
