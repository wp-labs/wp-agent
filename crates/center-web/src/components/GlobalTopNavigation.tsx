import { useEffect, useRef, useState, type FormEvent } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { NavLink } from "react-router-dom";
import {
  ADMIN_AUTH_CHANGED_EVENT,
  clearAdminApiToken,
  getAdminApiToken,
  setAdminApiToken,
} from "../api";
import styles from "./GlobalTopNavigation.module.css";

interface NavItem {
  to: string;
  label: string;
  end?: boolean;
}

const NAV_ITEMS: NavItem[] = [
  { to: "/", label: "网关态势", end: true },
  { to: "/instance", label: "网关管理", end: true },
  { to: "/release", label: "版本发布" },
  { to: "/upgrade-plan", label: "升级计划" },
  { to: "/upgrade-plan/approve", label: "批准升级计划" },
];

/** 渲染全局业务导航，并管理管理端 API Token 的本地应用状态。 */
export function GlobalTopNavigation() {
  const queryClient = useQueryClient();
  const [token, setToken] = useState(() => getAdminApiToken() ?? "");
  const [applied, setApplied] = useState(false);
  const appliedTimer = useRef<number | undefined>(undefined);

  useEffect(() => () => window.clearTimeout(appliedTimer.current), []);

  useEffect(() => {
    function refreshToken() {
      setToken(getAdminApiToken() ?? "");
    }
    window.addEventListener(ADMIN_AUTH_CHANGED_EVENT, refreshToken);
    return () => {
      window.removeEventListener(ADMIN_AUTH_CHANGED_EVENT, refreshToken);
    };
  }, []);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setAdminApiToken(token);
    setToken(getAdminApiToken() ?? "");
    setApplied(true);
    window.clearTimeout(appliedTimer.current);
    appliedTimer.current = window.setTimeout(() => setApplied(false), 1500);
    void queryClient.invalidateQueries();
  }

  function handleClear() {
    clearAdminApiToken();
    setToken("");
    void queryClient.invalidateQueries();
  }

  return (
    <header className={styles.container}>
      <div className={styles.inner}>
        <div className={styles.primary}>
          <div className={styles.brand} aria-label="WarpInsight 全局控制中心">
            <span className={styles.brandMark} aria-hidden="true">
              <span />
              <span />
              <span />
            </span>
            <span className={styles.brandCopy}>
              <strong>WarpInsight</strong>
              <small>Global Control Center</small>
            </span>
          </div>
          <nav className={styles.links} aria-label="主导航">
            {NAV_ITEMS.map((item) => (
              <NavLink
                key={item.to}
                className={({ isActive }) =>
                  isActive ? `${styles.link} ${styles.active}` : styles.link
                }
                to={item.to}
                end={item.end}
              >
                {item.label}
              </NavLink>
            ))}
          </nav>
        </div>
        <form className={styles.authForm} onSubmit={handleSubmit}>
          <label
            className={styles.authLabel}
            htmlFor="warp-insight-center-token"
          >
            <span className={styles.authDot} aria-hidden="true" />
            Admin Token
          </label>
          <input
            id="warp-insight-center-token"
            className={styles.authInput}
            type="password"
            autoComplete="off"
            placeholder="接入凭证"
            value={token}
            onChange={(event) => setToken(event.target.value)}
          />
          <button className={styles.authButtonPrimary} type="submit">
            {applied ? "已应用" : "应用"}
          </button>
          <button
            className={styles.authButton}
            type="button"
            onClick={handleClear}
            disabled={!token}
          >
            清除
          </button>
        </form>
      </div>
    </header>
  );
}
