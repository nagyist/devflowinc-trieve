import { createQuery } from "@tanstack/solid-query";
import { ChartCard } from "./ChartCard";
import { AnalyticsParams } from "shared/types";
import { createEffect, createSignal, onCleanup, useContext } from "solid-js";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { getLatency } from "../../api/latency";
import { Chart } from "chart.js";
import { lightFormat } from "date-fns";

function parseCustomDateString(dateString: string) {
  const [datePart, timePart] = dateString.split(" ");
  const [year, month, day] = datePart.split("-");
  const [hour, minute, second] = timePart.split(":");

  // Parse the fractional seconds and offset
  const [wholeSec] = second.split(".");

  // Construct an ISO 8601 compliant string
  const isoString = `${year}-${month}-${day}T${hour}:${minute}:${wholeSec}`;

  console.log(isoString);

  return new Date(isoString);
}

interface LatencyGraphProps {
  filters: AnalyticsParams;
}
export const LatencyGraph = (props: LatencyGraphProps) => {
  const dataset = useContext(DatasetContext);
  const [canvasElement, setCanvasElement] = createSignal<HTMLCanvasElement>();
  let chartInstance: Chart | null = null;
  const latencyQuery = createQuery(() => ({
    queryKey: [
      "latency",
      { filters: props.filters, dataset: dataset().dataset.id },
    ],
    queryFn: async () => {
      return await getLatency(props.filters, dataset().dataset.id);
    },
  }));

  createEffect(() => {
    const canvas = canvasElement();
    const data = latencyQuery.data;

    if (!canvas || !data) return;

    if (!chartInstance) {
      // Create the chart only if it doesn't exist
      chartInstance = new Chart(canvas, {
        type: "line",
        data: {
          labels: [],
          datasets: [
            {
              label: "Time",
              data: [],
              borderWidth: 1,
            },
          ],
        },
        options: {
          scales: {
            y: {
              beginAtZero: true,
            },
          },
        },
      });
    }

    // Update the chart data
    chartInstance.data.labels = data.map((point) =>
      lightFormat(
        new Date(parseCustomDateString(point.time_stamp)),
        "HH:mm:ss",
      ),
    );
    chartInstance.data.datasets[0].data = data.map(
      (point) => point.average_latency,
    );
    chartInstance.update();
  });

  onCleanup(() => {
    if (chartInstance) {
      chartInstance.destroy();
      chartInstance = null;
    }
  });

  return (
    <ChartCard width={3}>
      <canvas ref={setCanvasElement} class="h-full w-full" />
    </ChartCard>
  );
};
