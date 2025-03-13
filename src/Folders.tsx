import {
    Breadcrumb,
    BreadcrumbEllipsis,
    BreadcrumbItem,
    BreadcrumbLink,
    BreadcrumbList,
    BreadcrumbPage,
    BreadcrumbSeparator,
  } from "@/components/ui/breadcrumb"
import { Button } from "./components/ui/button"
import { File, FilePlus, FileSymlink, Folder, FolderPlus, FolderSymlink } from "lucide-react"
import { Fragment, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ScrollArea } from "./components/ui/scroll-area";

function Folders({ currentDevice }: { currentDevice: string }) {
    const [path, setPath] = useState<string[]>(["/"]);
    const [lists, setLists] = useState<string[]>([]);

    useEffect(() => {
        invoke("get_list", { deviceId: currentDevice, path: path.join("/") }).then((res) => {
            setLists(res as string[])
        })
    }, [path])

    return (
        <div>
            <div className="flex w-full justify-between h-10">
                <div className="w-1/2 border-l-2 border-r-2 border-b-2 border-border p-2 flex items-center">
                    <Breadcrumb>
                        <BreadcrumbList>
                            {path.map((name, index, arr) => {
                                const visibleCount = 3;
                                const isLast = index === arr.length - 1;
                                const shouldShowEllipsis = arr.length > visibleCount && index === 1;
                                const isVisible =
                                    index === 0 || isLast || (arr.length <= visibleCount) || index === arr.length - (visibleCount - 1);

                                return (
                                    <Fragment key={index}>
                                        {index > 0 && isVisible && <BreadcrumbSeparator />}
                                        {shouldShowEllipsis ? (
                                            <BreadcrumbEllipsis key="ellipsis" />
                                        ) : isVisible ? (
                                            isLast ? (
                                                <BreadcrumbPage>{name}</BreadcrumbPage>
                                            ) : (
                                                <BreadcrumbLink onClick={() => setPath(arr.slice(0, index + 1))} className="cursor-pointer">
                                                    {name}
                                                </BreadcrumbLink>
                                            )
                                        ) : null}
                                    </Fragment>
                                );
                            })}
                        </BreadcrumbList>
                    </Breadcrumb>
                </div>
                <div>
                    <Button variant="ghost" className="rounded-none" disabled>
                        <FolderPlus /> Add folder
                    </Button>
                    <Button variant="ghost" className="rounded-none" disabled>
                        <FilePlus /> Add file
                    </Button>
                </div>
            </div>
            <ScrollArea className="h-[calc(100vh-5rem-1px-0.50rem-2.5rem)]">
                <div className="grid grid-cols-5 lg:grid-cols-10">
                    {lists.map((name, i) => {
                        let icon;
                        if (name.endsWith("/")) {
                            icon = <Folder size={64}/>
                        }
                        else if (name.endsWith("@")) {
                            icon = <FolderSymlink size={64}/>
                        }
                        else {
                            icon = <File size={64}/>
                        }
                        
                        return (
                            <div className="flex flex-col items-center hover:bg-muted cursor-pointer aspect-square justify-center" key={`${name}-${i}`} onClick={() => {
                                    if (name.endsWith("/")) {
                                        setPath([...path, name.slice(0, -1)])
                                    }
                                }
                            }>
                                {icon}
                                <p>{name.slice(0, -1)}</p>
                            </div>
                        )
                    })}
                </div>
            </ScrollArea>
        </div>
    )
}

export default Folders