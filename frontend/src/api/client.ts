import type {
  BundleIngestResult,
  ObservationRow,
  PatientEntry,
  Timeline,
} from './types';

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`/api/v1${path}`, {
    ...init,
    headers: init?.body ? { 'Content-Type': 'application/json' } : undefined,
  });
  if (!res.ok) {
    const body = await res.text();
    throw new ApiError(res.status, body || res.statusText);
  }
  return (await res.json()) as T;
}

export function fetchPatients(): Promise<{ patients: PatientEntry[] }> {
  return api('/patients');
}

export function fetchTimeline(patientId: string): Promise<Timeline> {
  return api(`/patients/${encodeURIComponent(patientId)}/timeline`);
}

export function fetchObservations(
  patientId: string,
  loinc?: string,
): Promise<{ observations: ObservationRow[] }> {
  const query = loinc ? `?loinc=${encodeURIComponent(loinc)}` : '';
  return api(`/patients/${encodeURIComponent(patientId)}/observations${query}`);
}

export function postBundle(bundleJson: string): Promise<BundleIngestResult> {
  return api('/bundles', { method: 'POST', body: bundleJson });
}
