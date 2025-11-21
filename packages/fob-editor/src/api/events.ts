import type {
  EventAPI,
  EditorEvent,
  EventHandlers,
  Unsubscribe,
} from '../types/index.js';

/**
 * Implementation of the Event API
 * Provides pub/sub event system for editor events
 */
export class Events implements EventAPI {
  private handlers: Map<EditorEvent, Set<Function>> = new Map();

  on<E extends EditorEvent>(
    event: E,
    handler: EventHandlers[E]
  ): Unsubscribe {
    if (!this.handlers.has(event)) {
      this.handlers.set(event, new Set());
    }
    this.handlers.get(event)!.add(handler);

    // Return unsubscribe function
    return () => this.off(event, handler);
  }

  off<E extends EditorEvent>(event: E, handler: EventHandlers[E]): void {
    const handlers = this.handlers.get(event);
    if (handlers) {
      handlers.delete(handler);
    }
  }

  once<E extends EditorEvent>(
    event: E,
    handler: EventHandlers[E]
  ): Unsubscribe {
    const wrappedHandler = (
      (context: Parameters<EventHandlers[E]>[0]) => {
        handler(context);
        this.off(event, wrappedHandler as EventHandlers[E]);
      }
    ) as EventHandlers[E];

    return this.on(event, wrappedHandler);
  }

  emit<E extends EditorEvent>(
    event: E,
    context: Parameters<EventHandlers[E]>[0]
  ): void {
    const handlers = this.handlers.get(event);
    if (handlers) {
      handlers.forEach((handler) => {
        try {
          handler(context);
        } catch (error) {
          console.error(`Error in ${event} handler:`, error);
        }
      });
    }
  }

  /**
   * Clear all handlers for a specific event or all events
   */
  clear(event?: EditorEvent): void {
    if (event) {
      this.handlers.delete(event);
    } else {
      this.handlers.clear();
    }
  }

  /**
   * Get the number of handlers registered for an event
   */
  listenerCount(event: EditorEvent): number {
    return this.handlers.get(event)?.size ?? 0;
  }
}
