export interface PatientEntry {
  id: string;
  name: string;
  gender: string | null;
  birth_date: string | null;
}

export interface TimelineEvent {
  when: string | null;
  kind: 'observation' | 'condition' | 'medication';
  label: string;
  detail: string;
}

export interface Timeline {
  patient_id: string;
  events: TimelineEvent[];
}

export interface ObservationRow {
  loinc: { code: string; display: string } | null;
  effective: string | null;
  value: string;
  status: string;
}

export interface BundleIngestResult {
  bundle_type: string;
  entries: number;
  patients: number;
  observations: number;
  conditions: number;
  medications: number;
}
