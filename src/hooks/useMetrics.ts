import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { MetricsSummary, MetricsUpdate, RequestEntry } from "../types";

export function useMetrics(period: MetricsSummary["period"] = "day") {
  const [summary, setSummary] = useState<MetricsSummary | null>(null);
  const [recentRequests, setRecentRequests] = useState<RequestEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  const fetchSummary = useCallback(async () => {
    try {
      const data = await invoke<MetricsSummary>("get_metrics_summary", { period });
      setSummary(data);
    } catch {
      // Backend not ready yet
    } finally {
      setIsLoading(false);
    }
  }, [period]);

  useEffect(() => {
    fetchSummary();

    // Subscribe to real-time updates pushed from Rust backend
    const unlisten = listen<MetricsUpdate>("metrics_update", (event) => {
      setSummary(event.payload.summary);
      setRecentRequests(event.payload.recentRequests);
    });

    return () => {
      unlisten.then((unlistenFn: () => void) => unlistenFn());
    };
  }, [fetchSummary]);

  return { summary, recentRequests, isLoading, refresh: fetchSummary };
}
