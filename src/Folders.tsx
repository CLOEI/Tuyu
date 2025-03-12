import {
    Breadcrumb,
    BreadcrumbItem,
    BreadcrumbList,
    BreadcrumbPage,
  } from "@/components/ui/breadcrumb"
import { Button } from "./components/ui/button"
import { FilePlus, Folder, FolderPlus } from "lucide-react"
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function Folders({ currentDevice }: { currentDevice: string }) {
    const [path, setPath] = useState<string[]>(["/"]);
    const [folders, setFolders] = useState<string[]>([]);

    useEffect(() => {
        invoke("get_folders", { deviceId: currentDevice, path: path.join("/") })
    }, path)

    console.log(folders)
    return (
        <div>
            <div className="flex w-full justify-between">
                <div className="w-1/2 bg-muted p-2">
                    <Breadcrumb>
                        <BreadcrumbList>
                            <BreadcrumbItem>
                            <BreadcrumbPage>/</BreadcrumbPage>
                            </BreadcrumbItem>
                        </BreadcrumbList>
                    </Breadcrumb>
                </div>
                <div>
                    <Button variant="ghost" className="rounded-none">
                        <FolderPlus /> Add folder
                    </Button>
                    <Button variant="ghost" className="rounded-none">
                        <FilePlus /> Add file
                    </Button>
                </div>
            </div>
            {folders.map((name) => (
                <div>
                    <Folder/>
                    <p>{name}</p>
                </div>
            ))}
        </div>
    )
}

export default Folders