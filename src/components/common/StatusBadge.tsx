interface StatusBadgeProps {
  status: "healthy" | "degraded" | "down" | "unknown" | "success" | "error" | "fallback" | "timeout";
  label?: string;
}

const statusConfig = {
  healthy:  { dot: "bg-emerald-400", text: "text-emerald-400", label: "Activo" },
  success:  { dot: "bg-emerald-400", text: "text-emerald-400", label: "Éxito" },
  degraded: { dot: "bg-yellow-400",  text: "text-yellow-400",  label: "Degradado" },
  fallback: { dot: "bg-yellow-400",  text: "text-yellow-400",  label: "Fallback" },
  down:     { dot: "bg-red-400",     text: "text-red-400",     label: "Caído" },
  error:    { dot: "bg-red-400",     text: "text-red-400",     label: "Error" },
  timeout:  { dot: "bg-red-400",     text: "text-red-400",     label: "Timeout" },
  unknown:  { dot: "bg-slate-400",   text: "text-slate-400",   label: "Desconocido" },
};

export function StatusBadge({ status, label }: StatusBadgeProps) {
  const config = statusConfig[status] ?? statusConfig.unknown;
  return (
    <span className="inline-flex items-center gap-1.5">
      <span className={`w-2 h-2 rounded-full ${config.dot} animate-pulse`} />
      <span className={`text-xs font-medium ${config.text}`}>{label ?? config.label}</span>
    </span>
  );
}
