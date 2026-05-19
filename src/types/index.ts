// === Core domain types (mirroring Rust structs via Tauri IPC) ===
// SECURITY: Ningún tipo aquí debe contener API keys o material criptográfico.

export interface ProxyStatus {
  running: boolean;
  port: number;
  requestsTotal: number;
  startedAt: number | null;
}

export interface MetricsSummary {
  totalCostUsd: number;
  totalTokens: number;
  requestCount: number;
  successRate: number;
  avgLatencyMs: number;
  byProvider: ProviderMetrics[];
  period: "hour" | "day" | "week" | "month";
}

export interface ProviderMetrics {
  provider: string;
  requestCount: number;
  totalCostUsd: number;
  totalTokens: number;
  avgLatencyMs: number;
  errorRate: number;
  healthStatus: "healthy" | "degraded" | "down" | "unknown";
}

export interface RequestEntry {
  id: string;
  createdAt: number;
  projectTag: string;
  provider: string;
  model: string;
  promptTokens: number;
  completionTokens: number;
  costUsd: number;
  latencyMs: number;
  status: "success" | "error" | "fallback" | "timeout";
  routerDecision?: RouterDecision;
}

export interface RouterDecision {
  complexity: "low" | "medium" | "high";
  taskType: string;
  tauUsed: number;
  routeReason: string;
  estimatedCost: number;
  confidence: number;
}

export interface BudgetStatus {
  projectTag: string;
  spentUsd: number;
  limitUsd: number;
  percentUsed: number;
  locked: boolean;
  period: "daily" | "weekly" | "monthly";
  alert60Sent: boolean;
  alert85Sent: boolean;
}

export interface VirtualKeyInfo {
  id: string;
  name: string;
  projectTag: string;
  allowedModels: string[] | null;
  allowedProviders: string[] | null;
  expiresAt: number | null;
  createdAt: number;
  revoked: boolean;
}

export interface ProviderInfo {
  id: string;
  name: string;
  displayName: string;
  enabled: boolean;
  priority: number;
  healthStatus: "healthy" | "degraded" | "down" | "unknown";
  lastHealthCheck: number | null;
  hasApiKey: boolean;
}

export interface RoutingConfig {
  tau: number;
  ollamaRouterUrl: string;
  routerModel: string;
  classifierModel: string;
  complexityThresholdLow: number;
  complexityThresholdHigh: number;
  fallbackToLocal: boolean;
}

// === IPC Event payloads ===

export interface BudgetAlert {
  projectTag: string;
  percentUsed: number;
  level: "warning_60" | "warning_85" | "hard_lock";
  spentUsd: number;
  limitUsd: number;
}

export interface HealthChange {
  provider: string;
  previousStatus: string;
  newStatus: string;
  timestamp: number;
}

export interface MetricsUpdate {
  summary: MetricsSummary;
  recentRequests: RequestEntry[];
}
