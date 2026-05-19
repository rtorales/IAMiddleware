---
name: auditoria-cripto
description: >
  Ejecuta una auditoría completa de aislamiento criptográfico sobre el
  directorio src/ (frontend React). Detecta fugas de llaves API, tokens
  Bearer, llamadas HTTP directas a proveedores de IA y cualquier manejo
  de secretos en el contexto WebView. Ideal para correr antes de cada
  commit o merge.
disable-model-invocation: false
allowed-tools:
  - Grep
  - Read
  - Glob
context: fork
---

# Skill: Auditoría de Aislamiento Criptográfico

## Activación
Esta habilidad se activa al escribir `/auditoria-cripto` o cuando se
detecta que se han modificado archivos en `src/`.

## Proceso de ejecución

### Paso 1 — Listar archivos frontend
```
Glob: src/**/*.{ts,tsx,js,jsx}
```

### Paso 2 — Patrones de riesgo crítico (deben tener cero ocurrencias)
Grep en src/ por cada uno:
- `apiKey\s*[=:]` — asignación de apiKey en frontend
- `api\.openai\.com` — llamada directa a OpenAI
- `api\.anthropic\.com` — llamada directa a Anthropic
- `generativelanguage\.googleapis` — llamada directa a Gemini
- `Bearer\s+[A-Za-z0-9]` — token Bearer hardcodeado
- `sk-[a-zA-Z0-9-]{20,}` — patrón de API key OpenAI
- `fetch\(.*openai\|anthropic\|deepseek` — fetch directo a proveedores

### Paso 3 — Patrones de riesgo alto (revisar manualmente)
- `localStorage.*[Kk]ey\|[Ss]ecret\|[Tt]oken`
- `sessionStorage.*[Kk]ey\|[Ss]ecret`
- Imports de `crypto` en archivos `.tsx`

### Paso 4 — Emitir reporte

Formato de salida:
```
╔══════════════════════════════════════════════════════════╗
║         AUDITORÍA DE AISLAMIENTO CRIPTOGRÁFICO           ║
║         Fecha: [ISO datetime]                            ║
╠══════════════════════════════════════════════════════════╣
║ Archivos analizados: N                                   ║
║ Violaciones CRÍTICAS: N                                  ║
║ Advertencias ALTAS: N                                    ║
╠══════════════════════════════════════════════════════════╣
║ DETALLE:                                                 ║
║ [CRÍTICO] archivo:línea → fragmento                      ║
╠══════════════════════════════════════════════════════════╣
║ VEREDICTO: ✓ APROBADO / ✗ RECHAZADO                     ║
╚══════════════════════════════════════════════════════════╝
```

Si VEREDICTO es RECHAZADO: detener cualquier acción de commit/merge
y escalar al agente `rust-backend` para mover la lógica al backend.
