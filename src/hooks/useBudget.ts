import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { BudgetStatus, BudgetAlert } from "../types";

export function useBudget(projectTag?: string) {
  const [budgets, setBudgets] = useState<BudgetStatus[]>([]);
  const [alerts, setAlerts] = useState<BudgetAlert[]>([]);

  useEffect(() => {
    if (projectTag) {
      invoke<BudgetStatus>("get_budget_status", { projectTag })
        .then((b) => setBudgets([b]))
        .catch(() => {});
    } else {
      invoke<BudgetStatus[]>("list_budget_statuses")
        .then(setBudgets)
        .catch(() => {});
    }

    const unlisten = listen<BudgetAlert>("budget_alert", (event) => {
      setAlerts((prev) => [event.payload, ...prev].slice(0, 10));
    });

    return () => {
      unlisten.then((unlistenFn: () => void) => unlistenFn());
    };
  }, [projectTag]);

  const dismissAlert = (index: number) => {
    setAlerts((prev) => prev.filter((_, i) => i !== index));
  };

  return { budgets, alerts, dismissAlert };
}
