import { useState, useEffect } from "react";

const API_URL = "https://api.github.com/repos/opentaylor/taylorissue";

export function useGitHubStars() {
  const [stars, setStars] = useState<number | null>(null);

  useEffect(() => {
    let cancelled = false;

    fetch(API_URL)
      .then((res) => (res.ok ? res.json() : Promise.reject()))
      .then((data) => {
        if (!cancelled) setStars(data.stargazers_count ?? 0);
      })
      .catch(() => {});

    return () => {
      cancelled = true;
    };
  }, []);

  return stars;
}
