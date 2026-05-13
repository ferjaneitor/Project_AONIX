# 25 — Política de auditoría humana

> **Documento normativo.** Define cuándo y cómo se requiere intervención humana auditada en AONIX. Las reglas absolutas (R1, R2) **no son auditables**: son inmutables. Todo lo demás —reversión, degradación, importación, retiro, cambios de pruebas y excepciones de memoria— admite cambios solo bajo procedimientos formales con doble revisión y registro inmutable.

## Principio

> Auditoría humana = procedimiento formal con registro inmutable.
>
> No es "permiso de un superusuario", no es "modo god mode", no es atajo. Es un proceso documentado con responsable nombrado, justificación escrita, doble revisión, registro trazable y, cuando aplica, ventana de espera.

## Lo que **nunca** es auditable

Las siguientes reglas son **invariantes del sistema** y no admiten excepción por ninguna autoridad humana ni proceso de auditoría:

1. **R1** — El sistema es 2D.
2. **R2** — Las únicas primitivas son AND, OR, NOT.
3. **Principio rector** — La IA propone, AONIX (módulos deterministas) determina la verdad técnica.
4. **Unicidad del oficial activo** — Una sola versión oficial activa por (circuito, parámetros).
5. **Append-only de memoria histórica y de aprendizaje.**
6. **Atomicidad de la promoción.**
7. **El verificador es la única fuente de la decisión "correcto/incorrecto".**
8. **Inmutabilidad de un `.aonclg` cerrado.**

Cualquier intento de "auditar para cambiar" alguno de estos puntos se rechaza por construcción: no hay procedimiento que lo permita. Modificar uno de estos invariantes equivaldría a crear un sistema diferente, no a operar AONIX.

## Roles formales

| Rol | Responsabilidad |
|-----|-----------------|
| **Mantenedor técnico** (mt) | Propone cambios, aplica procedimientos, redacta justificaciones. |
| **Revisor independiente** (ri) | Revisa una propuesta del mt; debe ser una persona distinta. |
| **Aprobador final** (af) | Da el visto bueno definitivo; puede coincidir con el revisor o ser un tercer rol según severidad. |
| **Custodio de registros** (cr) | Garantiza que el registro de auditoría queda preservado; puede ser un sistema automatizado. |

Para el alcance inicial de AONIX (proyecto de un solo desarrollador), los roles pueden agruparse temporalmente, pero el procedimiento **registra los nombres** y el **doble check** sigue exigiéndose: ninguna decisión de auditoría se aplica sin que **dos sesiones explícitas** queden registradas (incluso si las protagoniza la misma persona, con margen temporal entre ellas).

Cuando AONIX crezca a un equipo, la separación de roles se hace estricta.

## Niveles de severidad

| Severidad | Ejemplos | Procedimiento requerido |
|-----------|---------|--------------------------|
| **S1 — Operativa** | Importación rutinaria de un circuito externo correctamente verificado; corrección de typo en metadatos de una tarea | mt + registro |
| **S2 — Sensible** | Reversión por regresión detectada; degradación controlada; eliminación de un caso límite catalogado | mt + ri + registro |
| **S3 — Crítica** | Retiro de oficial activo con `withdrawn`; cambio de regla operativa (no R1/R2); cambio de suite de pruebas con impacto histórico | mt + ri + af + registro + ventana de espera ≥ 24h |
| **S4 — Excepcional** | Eliminación de un `.aoncir` (raro); reescritura de memoria de optimización; cambio en convenciones de etiquetas que rompe compatibilidad | mt + ri + af + revisión cruzada + registro + ventana ≥ 7 días |
| **S0 — No auditable** | Cualquier intento de violar invariantes del sistema | **Rechazado por construcción** |

## Registro de auditoría

Cada acción de auditoría produce un **artefacto de auditoría inmutable** con:

```
audit_id:           UUID
severity:           S1 | S2 | S3 | S4
action_type:        (ver tabla más abajo)
target:             entidad afectada (hash canónico, task id, suite id, etc.)
proposed_at:        timestamp
proposed_by:        identidad del mt
reviewed_at:        timestamp
reviewed_by:        identidad del ri
approved_at:        timestamp        (si severidad ≥ S3)
approved_by:        identidad del af  (si severidad ≥ S3)
justification:      texto estructurado
evidence:           referencias (hashes, ids, snapshots)
expected_effect:    descripción del efecto deseado
applied_at:         timestamp efectivo
applied_by:         módulo de AONIX que ejecuta el cambio
rollback_plan:      pasos para revertir si fuese necesario
post_audit_check:   resultado de verificación post-aplicación
```

Los artefactos de auditoría se almacenan en una **memoria de auditoría** dedicada (puede considerarse una undécima memoria; ver [05 — Sistema de memorias](05-memory-system.md) §"Política de retención"). Esta memoria es **append-only** sin excepción y constituye la traza histórica de toda operación humana sobre AONIX.

---

## Procedimientos por tipo de acción

### A1 — Reversión administrativa

**Cuándo aplica.**

- Se detecta que una versión promovida como oficial activa no debería haberlo sido (bug del verificador, suite incompleta, modelo de referencia incorrecto).
- Se quiere restablecer una versión histórica previa como oficial activa.

**Severidad.** S3 — crítica.

**Procedimiento.**

1. **mt** redacta propuesta con:
   - Hash del oficial activo a revertir.
   - Hash de la versión histórica a re-activar.
   - Causa raíz (bug del verificador, suite, etc.).
   - Plan de re-verificación con suite ampliada antes de re-activar.
2. **ri** revisa: ¿la causa raíz se ha corregido en AONIX (verificador / suite)? Si no, **detiene** el procedimiento hasta corrección.
3. **af** aprueba.
4. Ventana de espera ≥ 24h durante la cual se ejecutan pruebas adicionales sobre la versión histórica candidata.
5. Aplicación: transacción atómica que mueve el oficial activo defectuoso a `withdrawn`, re-activa la histórica como oficial.
6. Registro inmutable.
7. Notificación a usuarios que tenían `.aonclg` activos referenciando el oficial revertido.

**Lo que la reversión nunca hace.**

- Borrar la versión defectuosa. Se marca `withdrawn`, no se elimina.
- Modificar el `.aoncir` defectuoso. Se conserva con metadata explícita.
- Borrar `.aonclg` que referenciaban la versión revertida.

### A2 — Degradación controlada

**Cuándo aplica.**

- Re-verificación periódica con suite ampliada hace fallar al oficial activo.
- Se descubre que un modelo de referencia era incompleto y el oficial activo falla la versión completa.

**Severidad.** S3 — crítica.

**Procedimiento.**

1. **mt** redacta propuesta con:
   - Hash del oficial activo a degradar.
   - Suite ampliada o modelo corregido como evidencia del fallo.
   - Casos específicos fallidos.
2. **ri** revisa: ¿el fallo es genuino o es un falso positivo de la nueva suite? Si falso positivo, **detiene**.
3. **af** aprueba.
4. Ventana de espera ≥ 24h.
5. Aplicación: el oficial activo se marca `degraded` (status del archivo). Sigue sirviéndose, pero con advertencia en el visualizador y traductores; se inicia búsqueda activa de reemplazo.
6. Registro.
7. Comunicación a agentes activos: episodios en curso reciben notificación; episodios futuros sobre tareas afectadas reciben aviso explícito.

**Diferencia con A1.** Degradación **no** restablece histórica; mantiene el oficial activo con marca de advertencia mientras se busca reemplazo válido.

### A3 — Importación de circuito

**Cuándo aplica.**

- Un humano o sistema externo aporta un `.aoncir` que se quiere considerar para promoción.
- Migración entre instalaciones de AONIX.

**Severidad.**

- S1 — operativa si el `.aoncir` pasa parser y verificador y va a memoria experimental.
- S2 — sensible si la importación se considera para reemplazar oficial activo.

**Procedimiento.**

1. **mt** prepara importación con `.aoncir` y referencia a tarea destino.
2. Parser estricto carga el `.aoncir`. Si falla L3, importación rechazada.
3. Verificador ejecuta suite completa. Si falla L1, importación rechazada.
4. Evaluador compara con oficial activo (si existe). Si no mejora estrictamente, queda en memoria experimental (S1 cerrada).
5. Si mejora, se genera propuesta de promoción S2: **ri** revisa que el `.aoncir` importado:
   - No introduce nodos prohibidos.
   - No introduce etiquetas semánticas fuera del catálogo.
   - Tiene metadatos consistentes (`author`, `created_at`, `imported_from`, `imported_by`).
6. **af** aprueba (si S2).
7. Promoción atómica normal.
8. Registro.

**Lo que la importación nunca hace.**

- Saltarse el parser, verificador o evaluador.
- Convertirse en oficial activo sin pasar las puertas.
- Heredar promoción de otra instalación sin re-verificación local.

### A4 — Retiro de versión oficial activa (`withdrawn`)

**Cuándo aplica.**

- Una versión oficial activa se considera incorrecta y no hay versión histórica adecuada para restablecer.
- Hallazgo de seguridad o de corrección que invalida el circuito de forma irrecuperable a corto plazo.

**Severidad.** S3 — crítica (S4 si no hay reemplazo posible y deja la tarea sin oficial activo).

**Procedimiento.**

1. **mt** redacta propuesta.
2. **ri** revisa la justificación y las consecuencias.
3. **af** aprueba.
4. Ventana de espera:
   - ≥ 24h si hay reemplazo candidato.
   - ≥ 7 días si la tarea quedará temporalmente sin oficial activo.
5. Aplicación:
   - El `.aoncir` se marca `withdrawn` con causa explícita.
   - Si hay reemplazo: promoción atómica de la candidata válida.
   - Si no hay reemplazo: la tarea queda sin oficial activo; los agentes que la intenten reciben aviso explícito.
6. Comunicación amplia.
7. Registro.

**Lo que el retiro nunca hace.**

- Borrar la versión retirada. Permanece con marca `withdrawn`.
- Eliminar `.aonclg` que la referenciaban. Quedan apuntando al `withdrawn`, con marca informativa.
- Reescribir historial.

### A5 — Cambios de reglas operativas (no R1/R2)

**Cuándo aplica.**

- Ajustes a umbrales: tasas mínimas de éxito, máximos de L0, ventanas de espera, factores de epsilon.
- Cambios en políticas de modos de estudio.
- Cambios en convenciones de visualización.
- Cambios en políticas de auditoría (este mismo documento).

**Severidad.** S3 — crítica para cambios con impacto en agentes en curso o en interpretación histórica; S2 para ajustes locales sin impacto histórico.

**Procedimiento.**

1. **mt** redacta propuesta versionada (semver del documento normativo afectado).
2. **ri** revisa impacto: ¿cambia retroactivamente la interpretación de algún `.aoncir`/`.aonclg`/auditoría previa?
3. **af** aprueba.
4. Ventana de espera ≥ 24h.
5. Aplicación: el documento normativo se publica con nueva versión; la versión anterior queda accesible.
6. Registro.
7. Comunicación.

**Lo que el cambio de reglas nunca hace.**

- Modificar R1 o R2 (no auditable, fuera del alcance de este procedimiento).
- Reescribir documentos previos (los anteriores quedan archivados con su `format_version` o `policy_version`).
- Cambiar retroactivamente decisiones (los `.aoncir`/`.aonclg` previos siguen interpretándose bajo la regla vigente al momento de su creación, salvo política de migración explícita).

### A6 — Cambios de pruebas

**Cuándo aplica.**

- Añadir un caso límite al catálogo: **no requiere auditoría** (append-only natural; los catálogos crecen).
- **Eliminar** un caso límite: requiere auditoría S2.
- Modificar la spec de una suite existente: requiere auditoría S3.
- Reescribir un modelo de referencia: requiere auditoría S3 (puede invalidar verificaciones históricas).

**Procedimiento (para eliminar o modificar).**

1. **mt** redacta propuesta con:
   - ID del caso/suite/modelo.
   - Causa raíz (caso obsoleto, redundante con otro, etc.).
   - Análisis de impacto: ¿algún oficial activo dependía de pasar este caso?
2. **ri** revisa.
3. **af** aprueba (S3).
4. Ventana de espera ≥ 24h.
5. Aplicación:
   - El caso/suite/modelo se marca `deprecated` con causa; no se elimina físicamente.
   - Las suites afectadas se versionan (nueva minor).
   - Re-verificación obligatoria de los oficiales activos afectados.
6. Si la re-verificación hace fallar algún oficial activo: se inicia A2 (degradación) o A4 (retiro) para esos circuitos.
7. Registro.

**Lo que el cambio de pruebas nunca hace.**

- Borrar evidencia histórica de fallos detectados por casos eliminados.
- Reescribir reportes de verificación pasados.
- Cambiar la suite sin re-verificar dependientes.

### A7 — Excepciones de memoria

**Cuándo aplica.**

- Reescritura de un layout en memoria visual por daño o corrupción.
- Reconstrucción de índices de memoria de aprendizaje.
- Operaciones administrativas sobre memoria experimental (consolidación, compresión, archivado a tier secundario).
- Operaciones sobre memoria de fallos para corregir clasificaciones erróneas.

**Severidad.**

- S1 — operativa para reindexaciones que no alteran contenido lógico.
- S2 — sensible para correcciones de clasificación o consolidaciones.
- S4 — excepcional para **eliminación** de cualquier entrada en cualquier memoria (sucede solo en casos extremos como pérdida de cumplimiento legal).

**Procedimiento.**

1. **mt** documenta la excepción con justificación detallada.
2. **ri** revisa.
3. **af** aprueba (si S2 o S4).
4. Ventana de espera según severidad.
5. Snapshot de la memoria afectada antes de aplicar.
6. Aplicación.
7. Registro inmutable con referencias al snapshot.

**Lo que la excepción de memoria nunca hace.**

- Eliminar la memoria de auditoría. Esa memoria es **inmutable absoluta**.
- Eliminar la memoria canónica de un oficial activo (eso es A4 — retiro).
- Cambiar contenido sin snapshot previo.

---

## Tabla resumen de procedimientos

| Acción | Severidad típica | mt | ri | af | Ventana | Registro |
|--------|------------------|----|----|----|---------|----------|
| Reversión administrativa (A1) | S3 | sí | sí | sí | ≥ 24h | inmutable |
| Degradación controlada (A2) | S3 | sí | sí | sí | ≥ 24h | inmutable |
| Importación a experimental (A3 caso 1) | S1 | sí | — | — | — | inmutable |
| Importación con promoción (A3 caso 2) | S2 | sí | sí | — | — | inmutable |
| Retiro `withdrawn` con reemplazo (A4) | S3 | sí | sí | sí | ≥ 24h | inmutable |
| Retiro `withdrawn` sin reemplazo (A4) | S4 | sí | sí | sí | ≥ 7 días | inmutable |
| Cambio de reglas operativas (A5) | S3/S2 | sí | sí | sí (S3) | ≥ 24h (S3) | inmutable |
| Añadir caso límite (A6) | — | sí | — | — | — | append-only natural |
| Eliminar caso límite (A6) | S2 | sí | sí | — | — | inmutable |
| Modificar suite/modelo (A6) | S3 | sí | sí | sí | ≥ 24h | inmutable |
| Excepción de memoria visual (A7) | S1 | sí | — | — | — | inmutable |
| Excepción consolidación memoria (A7) | S2 | sí | sí | — | — | inmutable |
| Eliminación en memoria (A7) | S4 | sí | sí | sí | ≥ 7 días | inmutable |
| Cambio de R1 o R2 | S0 | — | — | — | — | **NO PROCEDE** |

---

## Comunicación de cambios

Toda acción de auditoría con severidad ≥ S2 dispara comunicación a:

- **Usuarios humanos** con episodios o tareas activas afectadas.
- **Agentes de IA en entrenamiento** que pudieran haberse basado en un oficial activo ahora retirado: se les notifica en el siguiente `AgentState` que la información ha cambiado y se les ofrece la versión actualizada.
- **Memoria curricular**: si un agente avanzó dominando una tarea cuyo oficial activo se retira, el avance **no se anula** (es histórico), pero se registra que el dominio fue sobre una versión que ya no es vigente.

La comunicación no entrega el nuevo oficial activo dentro de un episodio en curso del agente si la tarea coincide: respeta las reglas de visibilidad (ver [16](16-ai-visibility-limits.md)).

## Casos no contemplados

Cuando una situación no encaja claramente en A1–A7, se aplica por defecto el procedimiento **S3** con doble revisión y ventana ≥ 24h, y la propia decisión se documenta como ampliación de este documento (que pasa por A5).

**No** existe "decisión sin registro" en AONIX. Toda intervención humana queda escrita.

## Auditoría sobre la auditoría

La memoria de auditoría es **inspeccionable** por cualquier usuario con permisos de lectura. Es **inmutable**. Una entrada errónea no se modifica; se añade una entrada de **corrección** que la referencia y explica. La cadena de correcciones queda visible.

Cambiar las reglas de auditoría (este documento) es operación A5 sobre el propio A5: cumple el procedimiento S3 estándar.

## Excepción explícita: bootstrap del sistema

Durante la **fase 0 actual** (proyecto recién iniciado, sin oficial activo, sin agentes en entrenamiento), las acciones administrativas son técnicamente operaciones del mantenedor único. El requisito formal del **doble check con margen temporal** sigue aplicando: ninguna decisión se aplica sin un **segundo paso reflexivo** registrado.

Cuando AONIX entre en operación (Fase 1 cerrada y siguientes), la separación estricta de roles se hace obligatoria.

## Garantías de la política de auditoría

1. **R1 y R2 son inmutables.** Ninguna autoridad humana las modifica.
2. **Todo cambio tiene registro inmutable.** La memoria de auditoría es append-only sin excepción.
3. **Doble revisión** para todo lo que no sea S1.
4. **Ventana de espera** evita decisiones impulsivas en casos críticos.
5. **Snapshots previos** garantizan reversibilidad técnica salvo en S0 (que no procede).
6. **Comunicación amplia** evita sorpresas a agentes y usuarios.
7. **Sin "modo god".** No existe usuario con privilegios para saltarse el procedimiento.

## Lo que la política **no** permite

- Modificar R1 o R2 bajo ningún procedimiento.
- Modificar la verdad técnica (decisión del verificador) por vía administrativa: si el verificador dijo FAIL, el circuito no se promueve. La auditoría puede corregir el **verificador** (vía A6), no anular su decisión sobre un caso pasado.
- Eliminar registros de auditoría.
- Eliminar `.aonclg` activos.
- Saltarse la verificación posterior a cualquier cambio que afecte oficiales activos.
- Aplicar cambios sin registro.
- Aplicar cambios sin justificación escrita.

## Decisiones cerradas en este documento

- Catálogo inicial de severidades (S0–S4).
- Catálogo de procedimientos A1–A7.
- Estructura del artefacto de auditoría.
- Ventanas de espera por severidad.
- Política de comunicación.

## Decisiones que siguen abiertas

- Implementación física de la memoria de auditoría (archivo plano, DB embebida, sistema externo de tickets).
- Mecanismo de firma criptográfica de artefactos de auditoría (opcional pero recomendado para integridad).
- Política de retención: ¿la memoria de auditoría tiene fecha de archivado a tier frío después de X años? Probablemente nunca, pero a confirmar.
- Cómo se materializa la "comunicación a agentes de IA" cuando esos agentes no son interactivos (decisión de canal: ¿señal en `AgentState`, evento separado?).
- Si la auditoría se delega total o parcialmente a herramientas automatizadas en Fases avanzadas (CI/CD podría ejercer parte del rol de **ri**).
