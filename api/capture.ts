import { handleCapturePost } from "./_lib/capture-core.js";

export async function POST(request: Request): Promise<Response> {
  const cfg = {
    captureKey: process.env.ARENA_CAPTURE_KEY ?? "",
    ghToken: process.env.ARENA_CAPTURE_GH_TOKEN ?? "",
    repo: process.env.ARENA_CAPTURE_REPO ?? "melnce/arena",
    branch: process.env.ARENA_CAPTURE_BRANCH ?? "captures",
  };
  return handleCapturePost(request, cfg, fetch);
}
