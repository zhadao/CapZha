import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { createMemo, createSignal, onCleanup, onMount, Show } from "solid-js";
import { t } from "~/components/I18nProvider";
import { commands } from "~/utils/tauri";

declare global {
	interface Window {
		__CAP_INSTANT_PROJECT_PATH__?: string;
		__CAP_INSTANT_FORMAT__?: string;
		__CAP_INSTANT_DELIVERY__?: string;
	}
}

const GIF_TIMEOUT_MS = 240_000;

export default function InstantPreview() {
	const projectPath = window.__CAP_INSTANT_PROJECT_PATH__ || "";
	const format = (window.__CAP_INSTANT_FORMAT__ || "mp4").toLowerCase();
	const deliveryPath = window.__CAP_INSTANT_DELIVERY__ || "";
	const isGif = format === "gif";

	const mp4Path = createMemo(() => `${projectPath}/content/output.mp4`);
	const [gifReady, setGifReady] = createSignal(!isGif);
	const [gifFailed, setGifFailed] = createSignal(false);
	const [showPrompt, setShowPrompt] = createSignal(true);
	const [message, setMessage] = createSignal("");
	const [copied, setCopied] = createSignal(false);

	const useGif = createMemo(() => isGif && !gifFailed());
	const activePath = createMemo(() =>
		useGif() ? deliveryPath : isGif ? mp4Path() : deliveryPath,
	);
	const activeSrc = createMemo(() =>
		activePath() ? convertFileSrc(activePath()) : "",
	);
	const ready = createMemo(() => !useGif() || gifReady());
	const fileName = createMemo(() => {
		const base = deliveryPath.split(/[/\\]/).pop();
		if (base) return base;
		return `cap.${format}`;
	});

	let pollTimer: number | undefined;
	let startedAt = 0;

	onMount(() => {
		if (!isGif || !deliveryPath) return;

		startedAt = Date.now();

		const check = async () => {
			try {
				if (await commands.instantDeliveryReady(deliveryPath)) {
					setGifReady(true);
					if (pollTimer) window.clearInterval(pollTimer);
					return;
				}
			} catch (e) {
				console.error("Failed to check GIF status:", e);
			}

			if (Date.now() - startedAt > GIF_TIMEOUT_MS) {
				setGifFailed(true);
				setMessage(t("recording.gifFailed"));
				if (pollTimer) window.clearInterval(pollTimer);
			}
		};

		void check();
		pollTimer = window.setInterval(check, 500);
	});

	onCleanup(() => {
		if (pollTimer) window.clearInterval(pollTimer);
	});

	const handleCopy = async () => {
		const path = activePath();
		if (!path || !ready()) return;

		try {
			await commands.copyInstantRecording(path);
			setCopied(true);
			setMessage(t("recording.copied"));
			setShowPrompt(false);
		} catch (e) {
			console.error("Copy failed:", e);
			setMessage(`${t("recording.copyFailed")}：${e}`);
		}
	};

	const handleExport = async () => {
		const path = activePath();
		if (!path || !ready()) return;

		try {
			const suggested = fileName().replace(/[\\/:*?"<>|]/g, "-");
			const savePath = await commands.saveFileDialog(suggested, "recording");
			if (!savePath) return;

			await commands.copyFileToPath(path, savePath);
			setMessage("导出成功");
		} catch (e) {
			console.error("Export failed:", e);
			setMessage(`导出失败：${e}`);
		}
	};

	const handleOpenFolder = async () => {
		const path = activePath();
		if (!path) return;
		await commands.openFilePath(path);
	};

	const handleClose = async () => {
		await commands.showWindow({ Main: { init_target_mode: null } });
		await getCurrentWindow().close();
	};

	return (
		<div class="flex h-screen flex-col bg-gray-50 text-gray-900 dark:bg-gray-900 dark:text-white">
			<div
				class="h-10 flex items-center justify-between px-4 bg-gray-100 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700"
				data-tauri-drag-region
			>
				<span class="text-sm font-medium">
					即时录制预览 · {format.toUpperCase()}
				</span>
				<button
					type="button"
					onClick={handleClose}
					class="w-6 h-6 flex items-center justify-center rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
				>
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M6 18L18 6M6 6l12 12"
						/>
					</svg>
				</button>
			</div>

			<Show when={showPrompt() && ready()}>
				<div class="flex items-center justify-between gap-4 px-4 py-2 bg-blue-50 dark:bg-blue-950 border-b border-blue-200 dark:border-blue-900">
					<span class="text-sm">{t("recording.copyPrompt")}</span>
					<div class="flex items-center gap-2">
						<button
							type="button"
							onClick={handleCopy}
							class="px-3 py-1.5 text-sm rounded bg-blue-500 hover:bg-blue-600 text-white font-medium transition-colors"
						>
							{t("recording.copy")}
						</button>
						<button
							type="button"
							onClick={() => setShowPrompt(false)}
							class="px-3 py-1.5 text-sm rounded bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
						>
							不用了
						</button>
					</div>
				</div>
			</Show>

			<div class="flex-1 flex items-center justify-center bg-gray-900 p-4 overflow-hidden">
				<Show
					when={ready()}
					fallback={
						<div class="flex flex-col items-center gap-3 text-gray-300">
							<div class="w-8 h-8 border-2 border-gray-500 border-t-transparent rounded-full animate-spin" />
							<p class="text-sm">{t("recording.gifGenerating")}</p>
						</div>
					}
				>
					<Show
						when={useGif()}
						fallback={
							<video
								src={activeSrc()}
								controls
								class="max-w-full max-h-full rounded-lg shadow-lg"
							/>
						}
					>
						<img
							src={activeSrc()}
							alt="GIF"
							class="max-w-full max-h-full object-contain rounded-lg shadow-lg"
						/>
					</Show>
				</Show>
			</div>

			<div class="h-16 flex items-center justify-between gap-4 px-4 bg-gray-100 dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700">
				<div class="min-w-0">
					<p class="text-xs text-gray-500 dark:text-gray-400 truncate">
						{fileName()}
					</p>
					<Show when={message()}>
						<p class="text-xs text-blue-500 dark:text-blue-400 truncate">
							{message()}
						</p>
					</Show>
				</div>
				<div class="flex items-center gap-2 shrink-0">
					<button
						type="button"
						onClick={handleOpenFolder}
						class="px-3 py-1.5 text-sm rounded bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
					>
						打开文件夹
					</button>
					<button
						type="button"
						onClick={handleExport}
						class="px-3 py-1.5 text-sm rounded bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
					>
						导出
					</button>
					<button
						type="button"
						onClick={handleCopy}
						disabled={!ready()}
						class="px-4 py-1.5 text-sm rounded font-medium text-white transition-colors disabled:opacity-50 bg-blue-500 hover:bg-blue-600"
					>
						{copied() ? t("recording.copied") : t("recording.copy")}
					</button>
				</div>
			</div>
		</div>
	);
}
