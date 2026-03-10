import { useState, useMemo } from "react";

export function usePagination<T>(items: T[], pageSize = 10) {
  const [page, setPage] = useState(0);
  const totalPages = Math.max(1, Math.ceil(items.length / pageSize));
  const safePage = Math.min(page, totalPages - 1);

  const pageItems = useMemo(
    () => items.slice(safePage * pageSize, (safePage + 1) * pageSize),
    [items, safePage, pageSize],
  );

  return {
    pageItems,
    page: safePage,
    totalPages,
    hasPrev: safePage > 0,
    hasNext: safePage < totalPages - 1,
    prev: () => setPage((p) => Math.max(0, p - 1)),
    next: () => setPage((p) => Math.min(totalPages - 1, p + 1)),
  };
}
