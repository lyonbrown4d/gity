import { useState } from "react";
import { useCustom, useCustomMutation } from "@refinedev/core";
import { useQueryClient } from "@tanstack/react-query";
import type { TodoDoneView, TodoState, TodoView } from "./todo-types";

const TODO_QUERY_KEY = ["todos"] as const;

export const useTodoList = (state: TodoState, limit = 100) => {
  const query = useCustom<TodoView[]>({
    url: "/todos",
    method: "get",
    config: { query: { state, limit } },
    queryOptions: {
      queryKey: [...TODO_QUERY_KEY, state, limit],
      refetchOnWindowFocus: false,
      retry: false,
    },
  });

  return {
    todos: Array.isArray(query.result.data) ? query.result.data : [],
    query: query.query,
  };
};

export const useMarkTodoDone = () => {
  const queryClient = useQueryClient();
  const [activeTodoId, setActiveTodoId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const { mutateAsync, mutation } = useCustomMutation<TodoDoneView>();

  const markDone = async (todoId: string): Promise<void> => {
    if (mutation.isPending) {
      return;
    }
    setActiveTodoId(todoId);
    setError(null);
    try {
      await mutateAsync({
        url: `/todos/${encodeURIComponent(todoId)}/done`,
        method: "post",
        values: {},
      });
      await queryClient.invalidateQueries({ queryKey: TODO_QUERY_KEY });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Failed to mark todo as done.");
    } finally {
      setActiveTodoId(null);
    }
  };

  return {
    markDone,
    activeTodoId,
    isPending: mutation.isPending,
    error,
  };
};
