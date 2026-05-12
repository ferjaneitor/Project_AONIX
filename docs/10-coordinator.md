# 10 — Coordinador central

## Naturaleza

El coordinador central **no es una IA**.

No inventa soluciones. No infiere. No "decide creativamente". Aplica reglas formales escritas, en orden determinista. Es la **orquesta** del mundo formal: enlaza validador, simulador, verificador, evaluador, memoria, currículo, visualizador y traductores en cada episodio.

Si una IA externa actúa como agente, el coordinador es el árbitro que la enfrenta al mundo. La IA propone, el coordinador hace cumplir.

## Responsabilidades

1. Recibir y cargar una tarea.
2. Cargar el nivel correspondiente y sus pruebas.
3. Cargar `.aonclg` previos si existen y son relevantes.
4. Iniciar el episodio con estado inicial canónico.
5. Recibir acciones de un agente (humano, búsqueda, IA).
6. Pasar cada acción al validador.
7. Si la acción es legal, actualizar el circuito parcial.
8. Si la acción es ilegal, devolver retroalimentación estructurada (sin modificar nada).
9. Ejecutar el simulador cuando una acción lo requiera (test de entrada, ejecución de batch, recálculo incremental).
10. Ejecutar el verificador al cierre o cuando el agente lo solicite explícitamente.
11. Ejecutar el evaluador para métricas.
12. Generar retroalimentación para IA y para humano (a través de los traductores).
13. Actualizar la visualización 2D.
14. Registrar cada paso en memoria de trayectorias y, al cierre, en `.aonclg`.
15. Decidir si el episodio continúa o termina, según condiciones formales.
16. Al cierre, si el circuito pasa el verificador, dispararse el flujo de optimización + verificación de la versión optimizada.
17. Si la versión final (optimizada o no) mejora a la oficial activa según el evaluador, ejecutar el reemplazo atómico:
    - Mover oficial activo a memoria histórica.
    - Promover nueva versión a oficial activa.
    - Actualizar índices.
    - Registrar en memoria de optimización.
18. Actualizar `.aonclg` con todos los datos del episodio (acciones, recompensas, resultados de verificación, deltas estructurales).
19. Actualizar memoria curricular si el dominio del nivel cambia.
20. Cerrar el episodio.

## Lo que el coordinador **no** hace

- **No invoca compuertas derivadas.** Si un agente propone usar XOR como primitiva, el coordinador no media: el validador rechaza y el coordinador devuelve la retroalimentación.
- **No modifica resultados** del verificador o del evaluador. Reporta lo que esos módulos producen.
- **No edita `.aoncir`** directamente. Solo a través del flujo verificación → optimización → reemplazo atómico.
- **No "interpreta" intenciones.** Solo acepta acciones explícitas, legales, enumerables.
- **No salta el validador** en ningún caso.
- **No declara correcto** un circuito sin el verificador.
- **No ajusta recompensas** post-hoc. La recompensa es función fija del estado.

## Flujo operativo completo

Este es el ciclo de vida de un episodio en AONIX.

```
1.  El usuario, currículo o agente externo define una tarea.
2.  AONIX convierte la tarea en una especificación formal (Task struct).
3.  Se carga el nivel correspondiente.
4.  Se cargan las pruebas proporcionales al nivel (memoria de pruebas).
5.  Se genera el estado inicial del episodio (circuito parcial vacío o
    semilla, señales disponibles iniciales, acciones legales iniciales).
6.  El agente propone una acción.
7.  El validador revisa la acción.
8.  Si es ILEGAL:
        - se rechaza,
        - se genera retroalimentación estructurada,
        - el estado del circuito no cambia,
        - se acumula penalización en la recompensa parcial,
        - se registra en memoria de fallos,
        - volver al paso 6 (mismo estado, agente decide siguiente acción).
9.  Si es LEGAL:
        - se actualiza el circuito parcial,
        - se computan nuevas acciones legales,
        - se acumula recompensa parcial,
        - se actualiza memoria de trayectorias.
10. Si la acción requiere simulación (test_specific_input, etc.), el
    simulador la ejecuta y devuelve el resultado al agente vía
    retroalimentación.
11. Si la acción es `stop_construction` o se alcanza una condición de
    cierre (límite de pasos, timeout, abandono):
        ir al paso 12.
    En otro caso, volver al paso 6.

12. CIERRE DEL EPISODIO (construcción):
    El coordinador toma el circuito parcial final y ejecuta:
        - simulador en batch para todos los casos del nivel,
        - verificador con las suites correspondientes,
        - evaluador con el conjunto de métricas activo.

13. RESULTADO DEL VERIFICADOR:
    - Si FALLA:
        - el circuito no entra en memoria canónica,
        - se registra como intento fallido en memoria experimental,
        - se acumula recompensa de cierre (probablemente cero o
          negativa),
        - se construye el `.aonclg` con la trayectoria y los detalles
          del fallo,
        - el episodio termina sin promoción.
    - Si PASA:
        ir al paso 14.

14. FASE DE OPTIMIZACIÓN:
    El coordinador invoca al optimizador estructural para producir una
    versión candidata mejorada (o devuelve la original si ya está en su
    forma minimal).

15. RE-VERIFICACIÓN DE LA VERSIÓN OPTIMIZADA:
    Se vuelve a ejecutar el verificador completo (correctitud no
    sacrificada).
    - Si la versión optimizada FALLA alguna prueba que la versión
      pre-optimizada superaba: la optimización se descarta y se
      conserva la versión pre-optimizada.
    - Si PASA: continúa con la optimizada.

16. COMPARACIÓN CON OFICIAL ACTIVO:
    - ¿Existe una versión oficial activa para este circuito y tamaño?
        - No: la nueva versión se promueve a oficial activa.
        - Sí: comparar usando el ranking del evaluador.
            - Si la nueva versión MEJORA estrictamente: promoción.
            - Si EMPATA: se conserva la oficial activa (estabilidad).
            - Si EMPEORA: la nueva no se promueve, queda en memoria
              experimental.

17. REEMPLAZO ATÓMICO (si procede):
    - mover oficial activo a memoria histórica,
    - escribir nueva versión como oficial activa,
    - actualizar índices,
    - registrar en memoria de optimización el delta,
    - notificar a memoria de aprendizaje para anotar el `.aonclg`.

18. ACTUALIZACIÓN DEL `.aonclg`:
    - meta del episodio,
    - trayectoria completa,
    - recompensas,
    - errores detectados,
    - resultados del verificador,
    - resultados del evaluador,
    - hash del `.aoncir` final,
    - referencia al hash del oficial activo al cierre,
    - bandera de reemplazo si aplica.

19. ACTUALIZACIÓN DE MEMORIA CURRICULAR:
    - si el agente avanzó significativamente en este nivel,
      actualizar progreso,
    - reevaluar condiciones de avance del nivel.

20. VISUALIZACIÓN FINAL:
    - render del circuito final,
    - render del antes/después si hubo optimización,
    - render del reemplazo si hubo promoción.

21. CIERRE LIMPIO:
    - todos los buffers persistidos,
    - el episodio queda inmutable en memoria de aprendizaje y de
      trayectorias,
    - el coordinador queda listo para el siguiente episodio.
```

## Criterios de cierre del episodio

El episodio termina cuando ocurre **cualquiera** de:

1. El agente emite `stop_construction`.
2. Se alcanza el límite de pasos del nivel (configurable, por defecto generoso).
3. Se alcanza el límite de tiempo del nivel.
4. Se detecta un loop de acciones inválidas (el agente intenta repetidamente acciones rechazadas sin avanzar).
5. Una condición de abandono explícito del agente.
6. Una excepción interna no recuperable de AONIX (raro; se trata como fallo de la plataforma, no del agente).

## Garantías del coordinador

1. **Determinismo:** misma tarea + mismo agente con misma semilla + mismo estado de memoria ⇒ misma trayectoria.
2. **Atomicidad del reemplazo:** la promoción de versión oficial activa es transaccional. O ocurre íntegra, o no ocurre.
3. **Inmutabilidad post-cierre:** el `.aonclg` de un episodio cerrado nunca se modifica.
4. **Trazabilidad:** cada paso queda registrado con timestamp, hash de estado, decisión de cada módulo.
5. **No cortocircuita módulos:** el coordinador no salta el validador, verificador, evaluador. Todo pasa por ellos.

## Errores y recuperación

Si un módulo del coordinador falla (excepción, panic, error de IO):

- El episodio se marca como **abortado por sistema**, no por el agente.
- El `.aonclg` correspondiente registra el aborto con el error.
- Memoria canónica **no se modifica** (transacciones atómicas garantizan esto).
- El coordinador se reinicia limpio para el siguiente episodio.
- El incidente se registra en auditoría.

Los abortos por sistema no penalizan al agente.

## Interfaz pública del coordinador

(Esquema, no firma definitiva.)

```
trait Coordinator {
    fn start_episode(task: Task) -> Episode;
    fn submit_action(episode: &mut Episode, action: Action) -> Feedback;
    fn request_visualization(episode: &Episode) -> RenderHint;
    fn close_episode(episode: Episode) -> EpisodeResult;
}
```

El agente externo solo ve esta superficie. Toda la complejidad interna (validador, simulador, verificador, evaluador, optimizador, memorias) queda detrás.

## Concurrencia y episodios paralelos

Múltiples episodios pueden correr en paralelo (entrenamiento masivo de IA, por ejemplo). El coordinador debe:

- Aislar el estado de cada episodio (sin estado global mutable compartido).
- Coordinar accesos a memoria canónica con bloqueo o concurrencia optimista.
- Asegurar que dos episodios que producen mejoras simultáneamente del mismo circuito se resuelven sin condición de carrera (uno gana atómicamente, el otro vuelve a comparar contra el nuevo oficial activo).
- Mantener determinismo por episodio (la concurrencia entre episodios no rompe el determinismo dentro de uno).

## Resumen

El coordinador es el árbitro. Conoce las reglas, las aplica sin desviarse, recoge lo que cada módulo produce y persiste el resultado. No interpreta. No infiere. No "es creativo". La creatividad pertenece al agente; la verdad pertenece a los módulos deterministas; el coordinador es quien los conecta.
