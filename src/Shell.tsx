import { useEffect, useRef } from "react";
import { ScrollArea } from "./components/ui/scroll-area"
import { invoke } from "@tauri-apps/api/core";

function Shell({ currentDevice }: { currentDevice: string }) {
    const logEndRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        invoke("hook_shell", { deviceId: currentDevice });
    }, [])

    return (
        <div>
            <ScrollArea className="h-[calc(100vh-5rem-1px-0.50rem)] w-full px-2">
                <div ref={logEndRef} />
            </ScrollArea>
        </div>
    )
}

export default Shell