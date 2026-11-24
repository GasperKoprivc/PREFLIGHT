export interface TeacherRoom {
  room_code: string;
  teacher_token: string;
  expires_at: number;
}
export interface SharedResult {
  check_id: string;
  status: "pass" | "fail" | "warning" | "unknown";
  installed_version: string | null;
  reason_code?: string | null;
}
export interface RoomResults {
  sessions: {
    session_id: string;
    alias: string;
    results: SharedResult[] | null;
  }[];
  aggregate: Record<string, number>;
  expires_at: number;
}
export class ApiError extends Error {
  constructor(public status: number) {
    super(
      status === 404
        ? "Soba je potekla ali je bila izbrisana."
        : status === 401
          ? "Učiteljska skrivnost ni veljavna."
          : status === 429
            ? "Preveč zahtev. Poskusi čez minuto."
            : "Zahteva ni uspela. Preveri profil ali povezavo.",
    );
  }
}
export async function api<T>(
  path: string,
  method = "GET",
  token?: string,
  body?: unknown,
): Promise<T> {
  const response = await fetch("/api/v1" + path, {
    method,
    headers: {
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...(body ? { "Content-Type": "application/json" } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
    signal: AbortSignal.timeout(10000),
    cache: "no-store",
    redirect: "error",
  });
  if (!response.ok) throw new ApiError(response.status);
  return response.status === 204 ? (undefined as T) : response.json();
}
export function restoreRoom(): TeacherRoom | null {
  try {
    const r = JSON.parse(sessionStorage.getItem("preflight-room") ?? "null");
    return r &&
      /^[A-HJ-NP-Z2-9]{8}$/.test(r.room_code) &&
      /^[a-f0-9]{64}$/.test(r.teacher_token) &&
      Number.isFinite(r.expires_at)
      ? r
      : null;
  } catch {
    return null;
  }
}
export const labels = {
  pass: "V redu",
  fail: "Napaka",
  warning: "Opozorilo",
  unknown: "Neznano",
};
