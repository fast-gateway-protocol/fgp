#!/usr/bin/env python3
"""
Generate benchmark visualization charts for FGP.

Outputs:
- docs/assets/benchmark-browser.svg (for README)
- docs/assets/benchmark-browser.png (for Twitter)
- docs/assets/benchmark-overhead.svg (cumulative overhead chart)
- docs/assets/benchmark-overhead.png
- docs/assets/benchmark-headline.png (hero image)
"""

import plotly.graph_objects as go
from plotly.subplots import make_subplots
import os
from pathlib import Path

# Output directory
ASSETS_DIR = Path(__file__).parent.parent / "docs" / "assets"
ASSETS_DIR.mkdir(parents=True, exist_ok=True)

# Dark theme colors (GitHub dark mode friendly)
COLORS = {
    "bg": "#0d1117",
    "card": "#161b22",
    "border": "#30363d",
    "text": "#e6edf3",
    "text_secondary": "#8b949e",
    "mcp": "#f85149",  # Red for slow
    "fgp": "#3fb950",  # Green for fast
    "accent": "#58a6ff",
}

# Benchmark data - ordered by speedup (highest first for visual impact)
BROWSER_DATA = {
    "operations": ["Navigate", "Snapshot", "Screenshot"],
    "mcp_ms": [2328, 2484, 1635],
    "fgp_ms": [8, 9, 30],
    "speedup": ["292×", "276×", "54×"],
}

# Light theme for Twitter
LIGHT_COLORS = {
    "bg": "#ffffff",
    "card": "#f6f8fa",
    "border": "#d0d7de",
    "text": "#1f2328",
    "text_secondary": "#656d76",
    "mcp": "#cf222e",
    "fgp": "#1a7f37",
    "accent": "#0969da",
}


def create_browser_benchmark_chart(colors):
    """Create a clean, impactful benchmark comparison chart."""

    operations = BROWSER_DATA["operations"]
    mcp_ms = BROWSER_DATA["mcp_ms"]
    fgp_ms = BROWSER_DATA["fgp_ms"]
    speedups = BROWSER_DATA["speedup"]

    fig = go.Figure()

    # Create grouped bars with MCP (slow) and FGP (fast)
    # MCP bars
    fig.add_trace(go.Bar(
        y=operations,
        x=mcp_ms,
        name="MCP Stdio",
        orientation="h",
        marker=dict(
            color=colors["mcp"],
            line=dict(width=0),
            cornerradius=4,
        ),
        text=[f"{ms:,}ms" for ms in mcp_ms],
        textposition="outside",
        textfont=dict(size=13, color=colors["text"], family="Arial Black"),
        cliponaxis=False,
    ))

    # FGP bars
    fig.add_trace(go.Bar(
        y=operations,
        x=fgp_ms,
        name="FGP Daemon",
        orientation="h",
        marker=dict(
            color=colors["fgp"],
            line=dict(width=0),
            cornerradius=4,
        ),
        text=[f"{ms}ms" for ms in fgp_ms],
        textposition="outside",
        textfont=dict(size=13, color=colors["fgp"], family="Arial Black"),
        cliponaxis=False,
    ))

    # Add speedup badges on the right (outside chart area)
    max_val = max(mcp_ms)
    badge_x = max_val + 650  # Move further right to avoid overlap
    for i, (op, speedup) in enumerate(zip(operations, speedups)):
        # Speedup badge background
        fig.add_shape(
            type="rect",
            x0=badge_x - 130,
            x1=badge_x + 130,
            y0=i - 0.35,
            y1=i + 0.35,
            fillcolor=colors["fgp"],
            line=dict(width=0),
            layer="below",
            xref="x",
            yref="y",
        )
        # Speedup text
        fig.add_annotation(
            x=badge_x,
            y=op,
            text=f"<b>{speedup}</b>",
            showarrow=False,
            font=dict(size=16, color="white", family="Arial Black"),
            xanchor="center",
        )

    fig.update_layout(
        title=dict(
            text="<b>FGP vs MCP: Browser Automation</b>",
            font=dict(size=22, color=colors["text"], family="Arial"),
            x=0.5,
            xanchor="center",
            y=0.95,
        ),
        barmode="group",
        bargap=0.25,
        bargroupgap=0.15,
        plot_bgcolor=colors["bg"],
        paper_bgcolor=colors["bg"],
        font=dict(color=colors["text"], size=14, family="Arial"),
        legend=dict(
            orientation="h",
            yanchor="bottom",
            y=1.02,
            xanchor="center",
            x=0.35,
            font=dict(size=13),
            bgcolor="rgba(0,0,0,0)",
        ),
        xaxis=dict(
            title=dict(
                text="Latency (ms) — lower is better",
                font=dict(size=12, color=colors["text_secondary"])
            ),
            tickfont=dict(size=11, color=colors["text_secondary"]),
            gridcolor=colors["border"],
            gridwidth=1,
            showgrid=True,
            zeroline=False,
            range=[0, max_val + 1000],  # Extended range for badges
            dtick=500,
        ),
        yaxis=dict(
            tickfont=dict(size=14, color=colors["text"], family="Arial"),
            showgrid=False,
            categoryorder="array",
            categoryarray=list(reversed(operations)),  # Navigate at top
        ),
        margin=dict(l=100, r=50, t=80, b=60),
        width=900,  # Wider to fit badges
        height=320,
    )

    return fig


def create_overhead_chart(colors):
    """Create line chart showing cumulative overhead over N tool calls."""

    tool_calls = list(range(0, 21))
    mcp_overhead_ms = 2300  # Average cold-start
    fgp_overhead_ms = 10    # Average daemon overhead

    mcp_cumulative = [n * mcp_overhead_ms / 1000 for n in tool_calls]
    fgp_cumulative = [n * fgp_overhead_ms / 1000 for n in tool_calls]

    fig = go.Figure()

    # Add shaded area between lines to emphasize the gap
    fig.add_trace(go.Scatter(
        x=tool_calls + tool_calls[::-1],
        y=mcp_cumulative + fgp_cumulative[::-1],
        fill="toself",
        fillcolor=f"rgba(248, 81, 73, 0.15)",  # MCP red with low opacity
        line=dict(width=0),
        showlegend=False,
        hoverinfo="skip",
    ))

    # MCP line
    fig.add_trace(go.Scatter(
        x=tool_calls,
        y=mcp_cumulative,
        name="MCP Stdio",
        mode="lines+markers",
        line=dict(color=colors["mcp"], width=3),
        marker=dict(size=5, color=colors["mcp"]),
    ))

    # FGP line
    fig.add_trace(go.Scatter(
        x=tool_calls,
        y=fgp_cumulative,
        name="FGP Daemon",
        mode="lines+markers",
        line=dict(color=colors["fgp"], width=3),
        marker=dict(size=5, color=colors["fgp"]),
    ))

    # Annotation at 20 calls for MCP
    fig.add_annotation(
        x=20,
        y=mcp_cumulative[-1],
        text="<b>46s wasted</b>",
        showarrow=True,
        arrowhead=2,
        arrowsize=1,
        arrowwidth=2,
        arrowcolor=colors["mcp"],
        font=dict(size=14, color=colors["mcp"], family="Arial Black"),
        ax=-60,
        ay=-35,
        bgcolor=colors["bg"],
        borderpad=4,
    )

    # Annotation for FGP
    fig.add_annotation(
        x=20,
        y=fgp_cumulative[-1] + 3,
        text="<b>0.2s total</b>",
        showarrow=False,
        font=dict(size=14, color=colors["fgp"], family="Arial Black"),
        bgcolor=colors["bg"],
        borderpad=4,
    )

    # Add "time saved" annotation in the middle
    fig.add_annotation(
        x=12,
        y=22,
        text="<b>Time saved</b>",
        showarrow=False,
        font=dict(size=16, color=colors["text_secondary"], family="Arial"),
    )

    fig.update_layout(
        title=dict(
            text="<b>Cold-Start Overhead Compounds</b>",
            font=dict(size=22, color=colors["text"], family="Arial"),
            x=0.5,
            xanchor="center",
            y=0.95,
        ),
        plot_bgcolor=colors["bg"],
        paper_bgcolor=colors["bg"],
        font=dict(color=colors["text"], size=14, family="Arial"),
        legend=dict(
            orientation="h",
            yanchor="bottom",
            y=1.02,
            xanchor="center",
            x=0.5,
            font=dict(size=13),
            bgcolor="rgba(0,0,0,0)",
        ),
        xaxis=dict(
            title=dict(
                text="Number of Tool Calls",
                font=dict(size=12, color=colors["text_secondary"])
            ),
            tickfont=dict(size=11, color=colors["text_secondary"]),
            gridcolor=colors["border"],
            showgrid=True,
            zeroline=False,
            dtick=5,
            range=[0, 21],
        ),
        yaxis=dict(
            title=dict(
                text="Cumulative Overhead (seconds)",
                font=dict(size=12, color=colors["text_secondary"])
            ),
            tickfont=dict(size=11, color=colors["text_secondary"]),
            gridcolor=colors["border"],
            showgrid=True,
            zeroline=False,
            range=[0, 52],
        ),
        margin=dict(l=70, r=70, t=80, b=60),
        width=800,
        height=400,
    )

    return fig


def create_headline_card(colors):
    """Create a bold headline card for Twitter."""

    fig = go.Figure()

    # Subtle gradient effect using shapes
    fig.add_shape(
        type="rect",
        x0=0, x1=1, y0=0, y1=1,
        xref="paper", yref="paper",
        fillcolor=colors["bg"],
        line=dict(width=0),
        layer="below",
    )

    # Add subtle border/glow effect
    fig.add_shape(
        type="rect",
        x0=0.05, x1=0.95, y0=0.1, y1=0.9,
        xref="paper", yref="paper",
        fillcolor=colors["card"],
        line=dict(color=colors["border"], width=1),
        layer="below",
    )

    # The big number
    fig.add_annotation(
        x=0.5,
        y=0.58,
        text="<b>292×</b>",
        showarrow=False,
        font=dict(size=140, color=colors["fgp"], family="Arial Black"),
        xref="paper",
        yref="paper",
    )

    # Subtitle
    fig.add_annotation(
        x=0.5,
        y=0.25,
        text="faster than MCP cold-start",
        showarrow=False,
        font=dict(size=26, color=colors["text_secondary"], family="Arial"),
        xref="paper",
        yref="paper",
    )

    # Brand
    fig.add_annotation(
        x=0.5,
        y=0.1,
        text="<b>FGP</b> — Fast Gateway Protocol",
        showarrow=False,
        font=dict(size=18, color=colors["accent"], family="Arial"),
        xref="paper",
        yref="paper",
    )

    # GitHub URL at bottom
    fig.add_annotation(
        x=0.5,
        y=0.02,
        text="github.com/fast-gateway-protocol",
        showarrow=False,
        font=dict(size=12, color=colors["text_secondary"], family="Arial"),
        xref="paper",
        yref="paper",
    )

    fig.update_layout(
        plot_bgcolor=colors["bg"],
        paper_bgcolor=colors["bg"],
        xaxis=dict(visible=False, range=[0, 1]),
        yaxis=dict(visible=False, range=[0, 1]),
        margin=dict(l=0, r=0, t=0, b=0),
        width=800,
        height=450,
    )

    return fig


def main():
    print("Generating FGP benchmark charts...\n")

    # Generate dark theme charts (for GitHub README)
    print("Creating dark theme charts...")

    browser_fig_dark = create_browser_benchmark_chart(COLORS)
    browser_fig_dark.write_image(ASSETS_DIR / "benchmark-browser.svg")
    browser_fig_dark.write_image(ASSETS_DIR / "benchmark-browser-dark.png", scale=2)
    print(f"  ✓ benchmark-browser.svg")
    print(f"  ✓ benchmark-browser-dark.png")

    overhead_fig_dark = create_overhead_chart(COLORS)
    overhead_fig_dark.write_image(ASSETS_DIR / "benchmark-overhead.svg")
    overhead_fig_dark.write_image(ASSETS_DIR / "benchmark-overhead-dark.png", scale=2)
    print(f"  ✓ benchmark-overhead.svg")
    print(f"  ✓ benchmark-overhead-dark.png")

    headline_fig_dark = create_headline_card(COLORS)
    headline_fig_dark.write_image(ASSETS_DIR / "benchmark-headline.svg")
    headline_fig_dark.write_image(ASSETS_DIR / "benchmark-headline-dark.png", scale=2)
    print(f"  ✓ benchmark-headline.svg")
    print(f"  ✓ benchmark-headline-dark.png")

    # Generate light theme charts (for Twitter)
    print("\nCreating light theme charts (for Twitter)...")

    browser_fig_light = create_browser_benchmark_chart(LIGHT_COLORS)
    browser_fig_light.write_image(ASSETS_DIR / "benchmark-browser-light.png", scale=2)
    print(f"  ✓ benchmark-browser-light.png")

    overhead_fig_light = create_overhead_chart(LIGHT_COLORS)
    overhead_fig_light.write_image(ASSETS_DIR / "benchmark-overhead-light.png", scale=2)
    print(f"  ✓ benchmark-overhead-light.png")

    headline_fig_light = create_headline_card(LIGHT_COLORS)
    headline_fig_light.write_image(ASSETS_DIR / "benchmark-headline-light.png", scale=2)
    print(f"  ✓ benchmark-headline-light.png")

    print(f"\n✅ All charts saved to {ASSETS_DIR}/")
    print("\nUsage in README.md:")
    print('  ![Browser Benchmark](docs/assets/benchmark-browser.svg)')


if __name__ == "__main__":
    main()
