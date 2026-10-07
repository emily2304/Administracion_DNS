import { supabase } from "./supabaseClient";

export async function getIpToCountry() {
  const { data, error } = await supabase
    .from("ip_to_country")
    .select("*")
    .order("country", { ascending: true });

  if (error) throw new Error(error.message);
  return data;
}

export async function createIpToCountry({ ipStart, ipEnd, country, city }) {
  const { data, error } = await supabase
    .from("ip_to_country")
    .insert({ ip_start: ipStart, ip_end: ipEnd, country, city: city || null })
    .select()
    .single();

  if (error) throw new Error(error.message);
  return data;
}

export async function updateIpToCountry(id, { ipStart, ipEnd, country, city }) {
  const { error } = await supabase
    .from("ip_to_country")
    .update({ ip_start: ipStart, ip_end: ipEnd, country, city: city || null })
    .eq("id", id);

  if (error) throw new Error(error.message);
}

export async function deleteIpToCountry(id) {
  const { error } = await supabase.from("ip_to_country").delete().eq("id", id);
  if (error) throw new Error(error.message);
}