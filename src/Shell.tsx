import { useEffect, useState } from "react";
import { ScrollArea } from "./components/ui/scroll-area"
import { invoke } from "@tauri-apps/api/core";
import { Terminal } from '@xterm/xterm';
import { listen } from "@tauri-apps/api/event";

function Shell({ currentDevice }: { currentDevice: string }) {
    const [terminal, setTerminal] = useState<Terminal | null>(null);
    const [output, setOutput] = useState<string>("");

    useEffect(() => {
        setTerminal(() => {
            let terminal = new Terminal()
            terminal.open(document.getElementById('terminal') as HTMLElement);

            return terminal;
        })
    }, [])

    useEffect(() => {
        if (terminal) {
            terminal.writeln("Connecting to device...");
            listen<string>("shell_output", (data) => {
                console.log(data.payload);
                setOutput(data.payload);
            });
            invoke("hook_shell", { deviceId: currentDevice });
        }
    }, [terminal])

    useEffect(() => {
        if (terminal) {
            terminal.writeln(output);
        }
    }, [output])

    return (
        <div>
            <ScrollArea className="h-[calc(100vh-5rem-1px-0.50rem)] w-full px-2">
                <div id="terminal"></div>
            </ScrollArea>
        </div>
    )
}

export default Shell