import { useCallback, useEffect, useState } from 'react';

import { fetchPatients, fetchTimeline, postBundle } from '@/api/client';
import type { PatientEntry, Timeline } from '@/api/types';

const KIND_STYLES: Record<string, { dot: string; label: string }> = {
  observation: { dot: 'bg-sky-500', label: 'Obs' },
  condition: { dot: 'bg-red-500', label: 'Cond' },
  medication: { dot: 'bg-emerald-500', label: 'Med' },
};

export default function App() {
  const [patients, setPatients] = useState<PatientEntry[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [timeline, setTimeline] = useState<Timeline | null>(null);
  const [bundleText, setBundleText] = useState('');
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchPatients()
      .then((body) => {
        setPatients(body.patients);
        setSelected((current) => current ?? body.patients[0]?.id ?? null);
      })
      .catch((err) => setError(String(err)));
  }, []);

  useEffect(() => {
    if (!selected) return;
    let cancelled = false;
    fetchTimeline(selected)
      .then((data) => {
        if (!cancelled) setTimeline(data);
      })
      .catch((err) => setError(String(err)));
    return () => {
      cancelled = true;
    };
  }, [selected]);

  const uploadBundle = useCallback(() => {
    if (!bundleText.trim()) return;
    setError(null);
    postBundle(bundleText)
      .then((result) => {
        setMessage(
          `Ingested ${result.entries} entries: +${result.patients} patients, ` +
            `+${result.observations} observations, +${result.conditions} conditions, ` +
            `+${result.medications} medications`,
        );
        setBundleText('');
        return fetchPatients();
      })
      .then((body) => setPatients(body.patients))
      .catch((err) => setMessage(String(err)));
  }, [bundleText]);

  return (
    <div className="flex h-full flex-col">
      <header className="border-b border-line bg-panel px-4 py-3">
        <h1 className="text-base font-semibold">FHIR R4 Data Explorer</h1>
        <p className="text-xs text-muted">
          Patient · Observation · Condition · MedicationStatement · Bundle —
          synthetic demo data
        </p>
      </header>

      <main className="grid flex-1 grid-cols-1 gap-4 overflow-y-auto p-4 lg:grid-cols-[1fr_3fr]">
        <aside className="flex flex-col gap-4">
          <section className="rounded-lg border border-line bg-panel p-3">
            <h2 className="mb-2 text-sm font-semibold">Patients</h2>
            <ul className="flex flex-col gap-1">
              {patients.map((patient) => (
                <li key={patient.id}>
                  <button
                    type="button"
                    onClick={() => setSelected(patient.id)}
                    className={`w-full rounded px-2 py-1 text-left text-xs hover:bg-surface ${
                      selected === patient.id ? 'bg-surface font-semibold' : ''
                    }`}
                  >
                    {patient.name}
                    <span className="ml-2 text-muted">
                      {patient.gender ?? '—'} · {patient.birth_date ?? '—'}
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </section>

          <section className="rounded-lg border border-line bg-panel p-3">
            <h2 className="mb-2 text-sm font-semibold">Ingest FHIR Bundle</h2>
            <p className="mb-2 text-[10px] text-muted">
              Paste a Bundle (collection) of modeled resources
            </p>
            <textarea
              value={bundleText}
              onChange={(e) => setBundleText(e.target.value)}
              rows={6}
              placeholder='{"resourceType":"Bundle","type":"collection","entry":[…]}'
              className="w-full rounded border border-line p-2 font-mono text-[11px]"
            />
            <button
              type="button"
              onClick={uploadBundle}
              disabled={!bundleText.trim()}
              className="mt-2 rounded bg-primary px-3 py-1 text-xs font-medium text-white disabled:opacity-50"
            >
              Ingest
            </button>
            {message && <p className="mt-2 text-xs text-muted">{message}</p>}
          </section>
        </aside>

        <section className="flex flex-col gap-2">
          {error && (
            <p className="rounded border border-red-300 bg-red-50 p-2 text-xs text-red-800">
              {error}
            </p>
          )}
          <h2 className="text-sm font-semibold">
            Longitudinal timeline — {selected ?? 'select a patient'}
          </h2>
          <ol className="flex flex-col gap-1">
            {timeline?.events.map((event, index) => {
              const style = KIND_STYLES[event.kind] ?? KIND_STYLES.observation;
              return (
                <li
                  key={index}
                  className="flex items-center gap-3 rounded border border-line bg-panel px-3 py-1.5 text-xs"
                >
                  <span className={`h-2.5 w-2.5 rounded-full ${style.dot}`} />
                  <span className="w-40 shrink-0 font-mono text-[10px] text-muted">
                    {event.when?.slice(0, 16).replace('T', ' ') ?? '—'}
                  </span>
                  <span className="w-10 shrink-0 text-[10px] uppercase text-muted">
                    {style.label}
                  </span>
                  <span className="font-medium">{event.label}</span>
                  <span className="text-muted">{event.detail}</span>
                </li>
              );
            })}
          </ol>
          {timeline && timeline.events.length === 0 && (
            <p className="text-xs text-muted">No modeled events for this patient.</p>
          )}
        </section>
      </main>
    </div>
  );
}
