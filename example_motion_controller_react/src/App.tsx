import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { LazyMotion, domAnimation } from "motion/react";
import * as m from "motion/react-m";

type AnimatedStyle = Record<string, number | string>;

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

function App() {
  return (
    <LazyMotion strict features={domAnimation}>
      <main className="page">
        <header className="hero" data-testid="controller-hero">
          <p className="eyebrow">Leptos Fluid Motion</p>
          <h1>AnimationController-only playground (React + Motion)</h1>
          <p className="lead">
            This app mirrors the controller-first demo with React and Motion,
            using tree-shaken imports for bundle-size comparison.
          </p>
        </header>

        <section className="grid">
          <ToggleCardExample />
          <TabsUnderlineExample />
          <PointerStateExample />
          <QueueLatestExample />
        </section>
      </main>
    </LazyMotion>
  );
}

function ToggleCardExample() {
  const [expanded, setExpanded] = useState(false);
  const [snapImmediate, setSnapImmediate] = useState(false);

  const cardStyle = useMemo(() => toggleCardStyle(expanded), [expanded]);

  const snapReset = useCallback(() => {
    setExpanded(false);
    setSnapImmediate(true);
    window.requestAnimationFrame(() => setSnapImmediate(false));
  }, []);

  return (
    <article className="panel" data-testid="controller-bind-panel">
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
        <m.div
          className="motion-card"
          data-testid="controller-bind-card"
          initial={false}
          animate={cardStyle}
          transition={
            snapImmediate
              ? { duration: 0 }
              : { type: "spring", duration: 0.56, bounce: 0.22 }
          }
        >
          <p className="chip">bind()</p>
          <h3>Controller as the animation primitive</h3>
          <p>
            No FluidDiv or FluidElement wrappers; state chooses styles and
            updates the target.
          </p>
        </m.div>
      </div>
    </article>
  );
}

function TabsUnderlineExample() {
  const [activeTab, setActiveTab] = useState(0);
  const [hoveredTab, setHoveredTab] = useState<number | null>(null);
  const [underlineImmediate, setUnderlineImmediate] = useState(true);
  const [underline, setUnderline] = useState({ x: 0, width: 0, opacity: 0 });

  const tabsRef = useRef<HTMLDivElement | null>(null);
  const tabRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const initialized = useRef(false);

  const measureTab = useCallback((index: number) => {
    const container = tabsRef.current;
    const tab = tabRefs.current[index];
    if (!container || !tab) {
      return null;
    }
    const containerRect = container.getBoundingClientRect();
    const tabRect = tab.getBoundingClientRect();
    return {
      x: tabRect.left - containerRect.left,
      width: tabRect.width,
      opacity: 1,
    };
  }, []);

  const highlightTab = hoveredTab ?? activeTab;

  useLayoutEffect(() => {
    const frame = window.requestAnimationFrame(() => {
      const next = measureTab(highlightTab);
      if (!next) {
        return;
      }

      setUnderline(next);

      if (!initialized.current) {
        initialized.current = true;
        setUnderlineImmediate(true);
        window.requestAnimationFrame(() => setUnderlineImmediate(false));
      }
    });

    return () => window.cancelAnimationFrame(frame);
  }, [highlightTab, measureTab]);

  useEffect(() => {
    const handleResize = () => {
      const next = measureTab(activeTab);
      if (next) {
        setUnderline(next);
      }
    };

    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, [activeTab, measureTab]);

  return (
    <article className="panel panel-wide" data-testid="controller-tabs-panel">
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

          <m.div
            className="tabs-underline"
            data-testid="controller-tab-underline"
            initial={false}
            animate={underline}
            transition={
              underlineImmediate
                ? { duration: 0 }
                : { type: "spring", duration: 0.52, bounce: 0.35 }
            }
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

  const style = useMemo(() => {
    if (pressed) {
      return pointerPressStyle(armed);
    }
    if (hovered) {
      return pointerHoverStyle(armed);
    }
    return pointerBaseStyle(armed);
  }, [armed, hovered, pressed]);

  const transition = useMemo(() => {
    if (pressed) {
      return { duration: 0.09 };
    }
    if (hovered) {
      return { duration: 0.14 };
    }
    return { duration: 0.18 };
  }, [hovered, pressed]);

  return (
    <article className="panel" data-testid="controller-pointer-panel">
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
        <m.button
          ref={pillRef}
          className={`control-pill ${armed ? "armed" : ""}`.trim()}
          data-testid="controller-pointer-pill"
          initial={false}
          animate={style}
          transition={transition}
        >
          {armed ? "Armed control" : "Idle control"}
        </m.button>
      </div>
    </article>
  );
}

function QueueLatestExample() {
  const [mounted, setMounted] = useState(false);
  const [queuedStep, setQueuedStep] = useState(0);

  const style = useMemo(() => queueStyle(queuedStep), [queuedStep]);

  return (
    <article className="panel" data-testid="controller-queue-panel">
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
          <m.div
            className="queue-chip"
            data-testid="controller-queue-chip"
            key={mounted ? "mounted" : "unmounted"}
            initial={{ opacity: 0.4, scale: 0.9, x: 0, y: 0, rotate: 0 }}
            animate={style}
            transition={{ type: "spring", duration: 0.48, bounce: 0.28 }}
          >
            <p className="chip">queued</p>
            <h3 data-testid="controller-queue-label">{queueLabel(queuedStep)}</h3>
          </m.div>
        ) : (
          <p className="detached-note" data-testid="controller-queue-detached">
            Detached: queue styles, then mount to replay latest.
          </p>
        )}
      </div>
    </article>
  );
}

function toggleCardStyle(expanded: boolean): AnimatedStyle {
  if (expanded) {
    return {
      opacity: 1,
      x: 0,
      y: 0,
      scale: 1,
      background: "linear-gradient(150deg, #0f766e, #155e75)",
      borderColor: "rgba(8, 145, 178, 0.65)",
      boxShadow: "0 22px 50px rgba(8, 47, 73, 0.28)",
    };
  }

  return {
    opacity: 0.82,
    x: -20,
    y: 12,
    scale: 0.94,
    background: "linear-gradient(160deg, #e7f4f1, #d8ebf8)",
    borderColor: "rgba(14, 116, 144, 0.24)",
    boxShadow: "0 10px 24px rgba(15, 23, 42, 0.14)",
  };
}

function pointerBaseStyle(armed: boolean): AnimatedStyle {
  if (armed) {
    return {
      scale: 1,
      boxShadow: "0 10px 20px rgba(8, 47, 73, 0.2)",
    };
  }

  return {
    scale: 1,
    boxShadow: "0 10px 20px rgba(15, 23, 42, 0.12)",
  };
}

function pointerHoverStyle(armed: boolean): AnimatedStyle {
  return {
    ...pointerBaseStyle(armed),
    scale: 1.04,
    boxShadow: "0 16px 28px rgba(15, 23, 42, 0.22)",
  };
}

function pointerPressStyle(armed: boolean): AnimatedStyle {
  return {
    ...pointerBaseStyle(armed),
    scale: 0.96,
    y: 2,
    boxShadow: "0 8px 14px rgba(15, 23, 42, 0.18)",
  };
}

function queueStyle(step: number): AnimatedStyle {
  switch (step % 4) {
    case 0:
      return {
        opacity: 0.76,
        x: -36,
        y: 10,
        rotate: -8,
        background: "linear-gradient(145deg, #dbeafe, #c7d2fe)",
      };
    case 1:
      return {
        opacity: 1,
        x: 0,
        y: 0,
        rotate: 0,
        scale: 1.02,
        background: "linear-gradient(145deg, #ccfbf1, #99f6e4)",
      };
    case 2:
      return {
        opacity: 0.96,
        x: 42,
        y: -14,
        rotate: 9,
        scale: 0.98,
        background: "linear-gradient(145deg, #fef3c7, #fde68a)",
      };
    default:
      return {
        opacity: 0.92,
        x: 14,
        y: 18,
        rotate: -4,
        scale: 1.06,
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

export default App;
