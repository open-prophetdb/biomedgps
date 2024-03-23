import { ChatModule, InitProgressReport } from "@mlc-ai/web-llm";
import { message } from "antd";

export const getJwtAccessToken = (): string | null => {
    let jwtToken = null;
    // Check if the cookie exists
    if (document.cookie && document.cookie.includes("jwt_access_token=")) {
        // Retrieve the cookie value
        // @ts-ignore
        jwtToken = document.cookie
            .split("; ")
            .find((row) => row.startsWith("jwt_access_token="))
            .split("=")[1];
    }

    if (jwtToken) {
        console.log("JWT access token found in the cookie.");
        return jwtToken;
    } else {
        console.log("JWT access token not found in the cookie.");
        return null;
    }
}

export const initChat = async () => {
    // const chat = new webllm.ChatWorkerClient(new Worker(
    //   new URL('./assets/web-llm.worker.js', import.meta.url),
    //   { type: 'module' }
    // ));
    const chat = new ChatModule();

    const remoteUrl = "https://huggingface.co";
    const genConfig = (url: string) => {
        let model_lib_baseurl = "https://raw.githubusercontent.com";
        if (url === window.location.origin) {
            model_lib_baseurl = url
        }

        const myAppConfig = {
            model_list: [
                {
                    "model_url": `${url}/mlc-ai/Mistral-7B-Instruct-v0.2-q4f16_1-MLC/resolve/main/`,
                    "local_id": "Mistral-7B-Instruct-v0.2-q4f16_1",
                    "model_lib_url": `${model_lib_baseurl}/mlc-ai/binary-mlc-llm-libs/main/Mistral-7B-Instruct-v0.2/Mistral-7B-Instruct-v0.2-q4f16_1-sw4k_cs1k-webgpu.wasm`,
                    "required_features": ["shader-f16"],
                },
            ]
        }

        return myAppConfig;
    }

    message.open({
        key: "loading",
        type: 'loading',
        content: 'Chat AI is loading...',
    });

    chat.setInitProgressCallback((report: InitProgressReport) => {
        if (report.progress === 1) {
            message.open({
                key: "loading",
                type: 'success',
                content: "Chat AI is loaded.",
                duration: 2,
            });
        } else {
            message.open({
                key: "loading",
                type: 'loading',
                content: `Chat AI is loading... ${(report.progress * 100).toFixed(2)}%`,
            })
        }
    });

    let appConfig = genConfig(remoteUrl);
    console.log("Chat AI is loading with remote config:", appConfig);
    await chat.reload("Mistral-7B-Instruct-v0.2-q4f16_1", undefined, appConfig);
    console.log("Chat AI is loaded.");

    // @ts-ignore
    window.chat = chat;

    return chat;
};
