import { supabase } from "./supabaseClient";

function limpiarIps(ips) {
  return ips.map((fila) => ({
    ...fila,
    ip: fila.ip.trim(),
    country: fila.country ? fila.country.trim().toUpperCase() : null,
    health_check: fila.health_check
      ? {
          ...fila.health_check,
          basic_auth_user: fila.health_check.basic_auth_user || null,
          basic_auth_pass: fila.health_check.basic_auth_pass || null,
        }
      : null,
  }));
}

export async function createRecord({ host, recordType, ips }) {
  const { data, error } = await supabase.rpc("create_dns_record", {
    p_host: host,
    p_record_type: recordType,
    p_ips: limpiarIps(ips),
  });

  if (error) throw new Error(error.message);
  return data;
}

export async function getRecords() {
  const { data, error } = await supabase
    .from("dns_records")
    .select(`
      id,
      host,
      record_type,
      created_at,
      dns_record_ips (
        id,
        ip,
        weight,
        country,
        is_healthy,
        rtt_ms,
        health_check_configs (
          check_type,
          port,
          http_path,
          expected_http_codes,
          basic_auth_user,
          basic_auth_pass,
          timeout_sec,
          retries,
          interval_sec
        )
      )
    `)
    .order("created_at", { ascending: false });

  if (error) throw new Error(error.message);
  return data;
}

export async function updateRecord(id, { host, recordType, ips }) {
  const { data, error } = await supabase.rpc("update_dns_record", {
    p_record_id: id,
    p_host: host,
    p_record_type: recordType,
    p_ips: limpiarIps(ips),
  });
  if (error) throw new Error(error.message);
  return data;
}

export async function deleteRecord(id) {
  const { error } = await supabase.from("dns_records").delete().eq("id", id);
  if (error) throw new Error(error.message);
}
