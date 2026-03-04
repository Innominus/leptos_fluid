import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useAnimate } from "motion/react-mini";

type CssTarget = Record<string, string | number>;

const TAB_LABELS = ["Overview", "Workflow", "Retargeting", "Queueing"];

const TAB_TITLES = [
  "Controller as execution layer",
  "App state chooses targets",
  "Mid-flight retargeting",
  "Queue latest semantics",
];

const TAB_BODIES = [
  "The underline is just a plain node with animate targets. There is no wrapper component requirement.",
  "App state owns measurements and selected indices, then sends x/width targets as declarative commands.",
  "Rapid tab clicks stay fluid because each new target starts from current visual progress.",
  "Detached targets can keep only the latest state and replay it when mounted.",
];

function AppMini() {
  return (
    <main className="page">
      <header className="hero" data-testid="controller-hero">
        <p className="eyebrow">Leptos Fluid Motion</p>
        <h1>AnimationController-only playground (React + Motion Mini)</h1>
        <p className="lead">
          This build mirrors the same demos using <code>useAnimate</code> from
          <code> motion/react-mini</code> for lean, imperative animation wiring.
        </p>
      </header>

      <section className="grid">
        <ToggleCardExample />
        <TabsUnderlineExample />
        <PointerStateExample />
        <QueueLatestExample />
      </section>
    </main>
  );
}

function ToggleCardExample() {
  const [expanded, setExpanded] = useState(false);
  const [snapImmediate, setSnapImmediate] = useState(false);
  const initialized = useRef(false);
  const [scope, animate] = useAnimate();

  useEffect(() => {
    const target = toggleCardStyle(expanded);
    const immediate = snapImmediate || !initialized.current;

    void animate("[data-testid='controller-bind-card']", target, {
      duration: immediate ? 0 : 0.56,
      ease: "easeOut",
    });

    initialized.current = true;
  }, [animate, expanded, snapImmediate]);

  const snapReset = useCallback(() => {
    setExpanded(false);
    setSnapImmediate(true);
    window.requestAnimationFrame(() => setSnapImmediate(false));
  }, []);

  return (
    <article className="panel" data-testid="controller-bind-panel" ref={scope}>
      <div className="panel-header">
        <h2>Declarative bind</h2>
        <p>Bind animation targets to app state and keep markup plain.</p>
      </div>

      <div className="button-row">
        <button
          data-testid="controller-bind-toggle"
          onClick={() => setExpanded((value) => !value)}
        >
          {expanded ? "Collapse" : "Expand"}
        </button>
        <button
          className="ghost"
          data-testid="controller-bind-reset"
          onClick={snapReset}
        >
          Snap reset
        </button>
      </div>

      <div className="stage">
        <div
          className="motion-card"
          data-testid="controller-bind-card"
          style={toggleCardStyle(false)}
        >
          <p className="chip">bind()</p>
          <h3>Controller as the animation primitive</h3>
          <p>
            No FluidDiv or FluidElement wrappers; state chooses styles and
            updates the target.
          </p>
        </div>
      </div>
    </article>
  );
}

function TabsUnderlineExample() {
  const [activeTab, setActiveTab] = useState(0);
  const [hoveredTab, setHoveredTab] = useState<number | null>(null);
  const [scope, animate] = useAnimate();
  const tabsRef = useRef<HTMLDivElement | null>(null);
  const tabRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const initialized = useRef(false);

  const highlightTab = hoveredTab ?? activeTab;

  const animateUnderline = useCallback(
    (index: number, immediate: boolean) => {
      const container = tabsRef.current;
      const tab = tabRefs.current[index];
      if (!container || !tab) {
        return;
      }

      const containerRect = container.getBoundingClientRect();
      const tabRect = tab.getBoundingClientRect();
      const x = tabRect.left - containerRect.left;

      void animate(
        "[data-testid='controller-tab-underline']",
        {
          transform: transformString(x, 0, 1, 0),
          width: `${tabRect.width}px`,
          opacity: 1,
        },
        {
          duration: immediate ? 0 : 0.52,
          ease: "easeOut",
        },
      );
    },
    [animate],
  );

  useLayoutEffect(() => {
    const frame = window.requestAnimationFrame(() => {
      animateUnderline(highlightTab, !initialized.current);
      initialized.current = true;
    });

    return () => window.cancelAnimationFrame(frame);
  }, [highlightTab, animateUnderline]);

  useEffect(() => {
    const onResize = () => animateUnderline(activeTab, true);
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, [activeTab, animateUnderline]);

  return (
    <article className="panel panel-wide" data-testid="controller-tabs-panel" ref={scope}>
      <div className="panel-header">
        <h2>Fluid tab underline</h2>
        <p>
          The underline is a plain element driven by one animation target.
          Rapid clicks retarget mid-flight without snapping.
        </p>
      </div>

      <div className="tabs-shell">
        <div
          className="tabs-list"
          ref={tabsRef}
          data-testid="controller-tabs-list"
          onPointerLeave={() => setHoveredTab(null)}
        >
          {TAB_LABELS.map((label, index) => (
            <button
              key={label}
              className={`tabs-button ${activeTab === index ? "active" : ""}`}
              ref={(node) => {
                tabRefs.current[index] = node;
              }}
              data-testid={`controller-tab-button-${index}`}
              onPointerEnter={() => setHoveredTab(index)}
              onClick={() => setActiveTab(index)}
            >
              {label}
            </button>
          ))}

          <div
            className="tabs-underline"
            data-testid="controller-tab-underline"
            style={{
              transform: transformString(0, 0, 1, 0),
              width: "0px",
              opacity: 0,
            }}
          />
        </div>

        <div className="tabs-content" data-testid="controller-tab-content">
          <h3>{TAB_TITLES[activeTab]}</h3>
          <p>{TAB_BODIES[activeTab]}</p>
        </div>
      </div>
    </article>
  );
}

function PointerStateExample() {
  const [armed, setArmed] = useState(false);
  const [hovered, setHovered] = useState(false);
  const [pressed, setPressed] = useState(false);
  const pillRef = useRef<HTMLButtonElement | null>(null);
  const [scope, animate] = useAnimate();

  useEffect(() => {
    const node = pillRef.current;
    if (!node) {
      return;
    }

    const onEnter = () => setHovered(true);
    const onLeave = () => {
      setHovered(false);
      setPressed(false);
    };
    const onDown = () => setPressed(true);
    const onUp = () => setPressed(false);
    const onCancel = () => {
      setHovered(false);
      setPressed(false);
    };

    node.addEventListener("pointerenter", onEnter);
    node.addEventListener("pointerleave", onLeave);
    node.addEventListener("pointerdown", onDown);
    node.addEventListener("pointerup", onUp);
    node.addEventListener("pointercancel", onCancel);

    return () => {
      node.removeEventListener("pointerenter", onEnter);
      node.removeEventListener("pointerleave", onLeave);
      node.removeEventListener("pointerdown", onDown);
      node.removeEventListener("pointerup", onUp);
      node.removeEventListener("pointercancel", onCancel);
    };
  }, []);

  const style = useMemo(() => pointerStyle(armed, hovered, pressed), [
    armed,
    hovered,
    pressed,
  ]);

  const duration = pressed ? 0.09 : hovered ? 0.14 : 0.18;

  useEffect(() => {
    void animate("[data-testid='controller-pointer-pill']", style, {
      duration,
      ease: "easeOut",
    });
  }, [animate, style, duration]);

  return (
    <article className="panel" data-testid="controller-pointer-panel" ref={scope}>
      <div className="panel-header">
        <h2>Manual interaction states</h2>
        <p>
          App logic handles hover/press/active priorities. Motion only applies
          the target style.
        </p>
      </div>

      <div className="button-row">
        <button
          className="ghost"
          data-testid="controller-pointer-arm-toggle"
          onClick={() => setArmed((value) => !value)}
        >
          {armed ? "Disable active mode" : "Enable active mode"}
        </button>
      </div>

      <div className="stage center">
        <button
          ref={pillRef}
          className={`control-pill ${armed ? "armed" : ""}`.trim()}
          data-testid="controller-pointer-pill"
          style={pointerStyle(false, false, false)}
        >
          {armed ? "Armed control" : "Idle control"}
        </button>
      </div>
    </article>
  );
}

function QueueLatestExample() {
  const [mounted, setMounted] = useState(false);
  const [queuedStep, setQueuedStep] = useState(0);
  const [scope, animate] = useAnimate();

  useEffect(() => {
    if (!mounted) {
      return;
    }

    void animate("[data-testid='controller-queue-chip']", queueStyle(queuedStep), {
      duration: 0.48,
      ease: "easeOut",
    });
  }, [animate, mounted, queuedStep]);

  return (
    <article className="panel" data-testid="controller-queue-panel" ref={scope}>
      <div className="panel-header">
        <h2>Queue latest while detached</h2>
        <p>
          Issue commands before mount, then attach the element and replay only
          the latest state.
        </p>
      </div>

      <div className="button-row">
        <button
          data-testid="controller-queue-next"
          onClick={() => setQueuedStep((step) => (step + 1) % 4)}
        >
          Queue next style
        </button>
        <button
          className="ghost"
          data-testid="controller-queue-mount"
          onClick={() => setMounted((value) => !value)}
        >
          {mounted ? "Unmount target" : "Mount target"}
        </button>
      </div>

      <div className="stage">
        {mounted ? (
          <div
            className="queue-chip"
            data-testid="controller-queue-chip"
            style={{
              opacity: 0.4,
              transform: transformString(0, 0, 0.9, 0),
              background: "linear-gradient(145deg, #e2e8f0, #cbd5e1)",
            }}
          >
            <p className="chip">queued</p>
            <h3 data-testid="controller-queue-label">{queueLabel(queuedStep)}</h3>
          </div>
        ) : (
          <p className="detached-note" data-testid="controller-queue-detached">
            Detached: queue styles, then mount to replay latest.
          </p>
        )}
      </div>
    </article>
  );
}

function transformString(x: number, y: number, scale: number, rotate: number) {
  return `translate3d(${x}px, ${y}px, 0px) scale(${scale}) rotate(${rotate}deg)`;
}

function toggleCardStyle(expanded: boolean): CssTarget {
  if (expanded) {
    return {
      opacity: 1,
      transform: transformString(0, 0, 1, 0),
      background: "linear-gradient(150deg, #0f766e, #155e75)",
      borderColor: "rgba(8, 145, 178, 0.65)",
      boxShadow: "0 22px 50px rgba(8, 47, 73, 0.28)",
    };
  }

  return {
    opacity: 0.82,
    transform: transformString(-20, 12, 0.94, 0),
    background: "linear-gradient(160deg, #e7f4f1, #d8ebf8)",
    borderColor: "rgba(14, 116, 144, 0.24)",
    boxShadow: "0 10px 24px rgba(15, 23, 42, 0.14)",
  };
}

function pointerStyle(armed: boolean, hovered: boolean, pressed: boolean): CssTarget {
  if (pressed) {
    return {
      transform: transformString(0, 2, 0.96, 0),
      boxShadow: "0 8px 14px rgba(15, 23, 42, 0.18)",
    };
  }

  if (hovered) {
    return {
      transform: transformString(0, 0, 1.04, 0),
      boxShadow: "0 16px 28px rgba(15, 23, 42, 0.22)",
    };
  }

  if (armed) {
    return {
      transform: transformString(0, 0, 1, 0),
      boxShadow: "0 10px 20px rgba(8, 47, 73, 0.2)",
    };
  }

  return {
    transform: transformString(0, 0, 1, 0),
    boxShadow: "0 10px 20px rgba(15, 23, 42, 0.12)",
  };
}

function queueStyle(step: number): CssTarget {
  switch (step % 4) {
    case 0:
      return {
        opacity: 0.76,
        transform: transformString(-36, 10, 1, -8),
        background: "linear-gradient(145deg, #dbeafe, #c7d2fe)",
      };
    case 1:
      return {
        opacity: 1,
        transform: transformString(0, 0, 1.02, 0),
        background: "linear-gradient(145deg, #ccfbf1, #99f6e4)",
      };
    case 2:
      return {
        opacity: 0.96,
        transform: transformString(42, -14, 0.98, 9),
        background: "linear-gradient(145deg, #fef3c7, #fde68a)",
      };
    default:
      return {
        opacity: 0.92,
        transform: transformString(14, 18, 1.06, -4),
        background: "linear-gradient(145deg, #ffedd5, #fed7aa)",
      };
  }
}

function queueLabel(step: number): string {
  switch (step % 4) {
    case 0:
      return "Queued: cool start";
    case 1:
      return "Queued: anchor";
    case 2:
      return "Queued: drift";
    default:
      return "Queued: flare";
  }
}

export default AppMini;
