import { getCurrentWindow, PhysicalPosition } from '@tauri-apps/api/window';
import "./App.css";
import "./xterm.css";
import { Button } from '@/components/ui/button';
import { Box, Cast, Settings, Smartphone, X } from 'lucide-react';
import { Separator } from '@/components/ui/separator';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { FaAndroid, FaJava } from "react-icons/fa";
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useEffect, useRef, useState } from 'react';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { ScrollArea } from './components/ui/scroll-area';
import Folders from './Folders';
import Shell from './Shell';

const appWindow = getCurrentWindow();

type AppDetail = {
  name: string;
  package_name: string;
  version: string;
  min_sdk: string;
  target_sdk: string;
  is_32bit: boolean;
  is_64bit: boolean;
  icon_base64?: string | null;
}

type Log = {
  time: string;
  message: string;
}

type Device = {
  id: string;
  model: string;
  state: string;  
}

function App() {
  const [appDetail, setAppDetail] = useState<AppDetail | null>(null);
  const [devices, setDevices] = useState<Device[] | null>(null);
  const [device, setDevice] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [javaPath, setJavaPath] = useState<string | null>(null);
  const [appPath, setAppPath] = useState<string | null>(null);
  const [aboutOpen, setAboutOpen] = useState(false);
  const [devicesOpen, setDevicesOpen] = useState(false);
  const [log, setLog] = useState<Log[]>([]);
  const [currentNav, setCurrentNav] = useState("utils");
  const isDragAndDropListenerRegistered = useRef(false);
  const isLogListenerRegistered = useRef(false);
  const logEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<Device[]>("get_adb_devices").then((data) => {
      setDevices(data);
      setDevice(data[0].id);
    });
  }, [])
  
  useEffect(() => {
    let unlistenDragAndDrop: UnlistenFn;

    (async () => {
      if (isDragAndDropListenerRegistered.current) return;
      isDragAndDropListenerRegistered.current = true;
      unlistenDragAndDrop = await listen<{ paths: string[], position: PhysicalPosition }>("tauri://drag-drop", (e) => {
        setAppPath(e.payload.paths[0]);
        setLog([])
        invoke<AppDetail>("get_app_detail", { appPath: e.payload.paths[0] }).then((data) => {
          setLog((prev) => [...prev, { time: new Date().toLocaleTimeString(), message: `App detail fetched for ${data.name}` }]);
          setAppDetail(data);
        })
      })
    })();

    if (!isLogListenerRegistered.current) {
      isLogListenerRegistered.current = true;
      listen<string>("log", (e) => {
        setLog((prev) => [...prev, { time: new Date().toLocaleTimeString(), message: e.payload }]);
      })
    }

    invoke<string|null>("get_java").then((path) => {
      if (path) {
        setLoading(false);
        setJavaPath(path);
      }
    })

    return () => {
      if (unlistenDragAndDrop) {
        unlistenDragAndDrop();
        isDragAndDropListenerRegistered.current = false;
      }
    }
  }, []);

  useEffect(() => {
    if (logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [log]);

  useEffect(() => {
    if (devicesOpen) {
      invoke<Device[]>("get_adb_devices").then((data) => {
        setDevices(data);
      })
    }
  }, [devicesOpen])

  return (
    <main className='p-1 flex flex-col h-screen'>
      <div className="w-full flex justify-between h-10 flex-shrink-0" onMouseDown={(e) => {
        if (e.buttons === 1 && !(e.target instanceof HTMLButtonElement)) {
          e.detail === 2
            ? appWindow.toggleMaximize()
            : appWindow.startDragging();
        }
      }}>
        <div className='flex items-center space-x-1'>
          <h1 className='font-mono'>TUYU</h1>
          <Button variant="ghost" size="sm" className='font-mono' onClick={() => { setAboutOpen(true) }}>
            About
          </Button>
          <Button variant="ghost" size="sm" className='font-mono' disabled>
            Help
          </Button>
        </div>
        <div>
          <Button onClick={() => { appWindow.close(); }} size="icon" variant="ghost" className='hover:bg-red-500'>
            <X />
          </Button>
        </div>
      </div>
      <div className='flex items-center justify-between h-10 flex-shrink-0'>
        <div className='flex items-center space-x-1'>
          <Select onValueChange={(val) => setDevice(val)} value={device}>
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder={(devices && devices.length > 0) ? "Choose a device" : "Device not connected"}/> 
            </SelectTrigger>
            <SelectContent>
              {devices && devices.length > 0 && devices.map((device) => {
                return (
                  <SelectItem value={device.id} key={device.id}>{device.model}</SelectItem>
                )
              })}
            </SelectContent>
          </Select>
          <Button size="icon" variant="ghost" onClick={() => setDevicesOpen(true)} disabled>
            <Smartphone />
          </Button>
          <Separator orientation='vertical' className='h-4'/>
          <Button size="icon" variant="ghost" disabled={device.length === 0} onClick={() => invoke("execute_scrcpy", { deviceId: device })}>
            <Cast />
          </Button>
          <Button variant="ghost" size="sm" className='font-mono' onClick={() => setCurrentNav("utils")}>
            Utils
          </Button>
          <Button variant="ghost" size="sm" className='font-mono' onClick={() => setCurrentNav("folders")} disabled={device.length === 0}>
            Folders
          </Button>
          <Button variant="ghost" size="sm" className='font-mono' onClick={() => setCurrentNav("logcat")} disabled={device.length === 0}>
            Logcat
          </Button>
          <Button variant="ghost" size="sm" className='font-mono' onClick={() => setCurrentNav("shell")} disabled={device.length === 0}>
            Shell
          </Button>
        </div>
        <div>
          <Button size="icon" variant="ghost" disabled>
            <Settings />
          </Button>
        </div>
      </div>
      <Separator className='mt-1'/>
      {currentNav == "utils" && (
        <>
          {!appDetail ? (
            <div className='h-full flex-1 flex items-center justify-center'>
              <div className='flex flex-col items-center'>
                {loading ? (
                  <>
                    <Box size={100} />
                    <p className='text-muted-foreground'>Checking binaries...</p>
                  </>
                ) : (
                  <>
                      <>
                        <FaAndroid size={100} />
                        <p className='text-muted-foreground'>Drag and drop folder, apk or xapk here</p>
                      </>
                  </>
                )}
              </div>
            </div>
            ) : (
              <div className='grid grid-cols-[50%,50%] w-full h-full'>
                <div>
                  {appDetail.icon_base64 && (
                    <img src={`data:image/png;base64,${appDetail.icon_base64}`} alt="App Icon" className='w-16 h-16 mr-4 rounded-md'/>
                  )}
                  <div className='flex flex-col space-y-2'>
                    <div className='grid grid-cols-2 gap-4'>
                      <div>
                        <p className='font-semibold'>Name:</p>
                        <p>{appDetail.name}</p>
                      </div>
                      <div>
                        <p className='font-semibold'>Package Name:</p>
                        <p>{appDetail.package_name}</p>
                      </div>
                    </div>
                    <div className='grid grid-cols-2 gap-4'>
                      <div>
                        <p className='font-semibold'>Min SDK:</p>
                        <p>{appDetail.min_sdk}</p>
                      </div>
                      <div>
                        <p className='font-semibold'>Target SDK:</p>
                        <p>{appDetail.target_sdk}</p>
                      </div>
                    </div>
                    <div className='grid grid-cols-2 gap-4'>
                      <div>
                        <p className='font-semibold'>Armeabi-v7a:</p>
                        <p>{appDetail.is_32bit ? "Yes" : "No"}</p>
                      </div>
                      <div>
                        <p className='font-semibold'>Arm64-v8a:</p>
                        <p>{appDetail.is_64bit ? "Yes" : "No"}</p>
                      </div>
                    </div>
                    <div>
                      <p className='font-semibold'>Version:</p>
                      <p>{appDetail.version}</p>
                    </div>
                  </div>
                  <div className='mt-4 flex flex-wrap gap-2'>
                    <Button onClick={() => {
                      setLog((prev) => [...prev, { level: "Info", time: new Date().toLocaleTimeString(), message: "Starting decompilation..." }]);
                      invoke("extract_app", { appPath: appPath, name: `${appDetail.name}-${appDetail.version}` })
                    }} disabled={!(appPath?.includes(".apk") || !appPath?.includes(".xapk") && appPath?.includes(".apk"))}>
                      Decompile
                    </Button>
                    <Button onClick={() => {
                      setLog((prev) => [...prev, { level: "Info", time: new Date().toLocaleTimeString(), message: "Starting compilation..." }]);
                      invoke("compile_app", { appPath: appPath, name: `${appDetail.name}-${appDetail.version}` })
                    }} disabled={!!(appPath?.includes(".apk") || appPath?.includes(".xapk"))}>
                      Compile
                    </Button>
                    <Button onClick={() => { 
                      setLog((prev) => [...prev, { level: "Info", time: new Date().toLocaleTimeString(), message: "Starting signing..." }]);
                      invoke("sign_apk", { apkPath: appPath })
                    }} disabled={!(appPath?.includes(".apk") && !appPath?.includes(".xapk"))}>
                      Sign
                    </Button>
                    <Button onClick={() => { 
                      setLog((prev) => [...prev, { level: "Info", time: new Date().toLocaleTimeString(), message: "Starting merge..." }]);
                      invoke("merge_xapk", { xapkPath: appPath, name: `${appDetail.name}-${appDetail.version}` })
                    }} disabled={!(appPath?.includes(".xapk"))}>
                      Merge xapk to apk
                    </Button>
                    <Button onClick={() => { console.log("Installing...") }} disabled={!(appPath?.includes(".apk") && !appPath?.includes(".xapk"))}>
                      Install
                    </Button>
                  </div>
                </div>
                <ScrollArea className="h-[calc(100vh-5rem-1px-0.50rem)] w-full border border-l-1 px-2">
                  {log.map((log, index) => (
                    <div key={index} className='grid grid-cols-[auto_auto_1fr] gap-x-2 items-start'>
                      <span className='text-muted-foreground whitespace'>[{log.time}] </span>
                      <span className='break-all'>{log.message}</span>
                    </div>
                  ))}
                  <div ref={logEndRef} />
                </ScrollArea>
              </div>
            )
          }
        </>
      )}
      {currentNav == "folders" && <Folders currentDevice={device}/>}
      {currentNav == "shell" && <Shell currentDevice={device}/>}
      <AlertDialog open={loading == false && javaPath == null}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Binary not found</AlertDialogTitle>
            <AlertDialogDescription>
                <div className='flex flex-col items-center space-y-2'>
                <FaJava size={64} />
                <span className='text-muted-foreground font-mono text-center'>Java is not installed. Please install Java to proceed.</span>
                <Button variant="outline" onClick={() => { openUrl("https://www.java.com/en/download/") }}>
                  Download Java
                </Button>
                </div>
            </AlertDialogDescription>
          </AlertDialogHeader>
        </AlertDialogContent>
      </AlertDialog>
      <AlertDialog open={aboutOpen} onOpenChange={(open) => { setAboutOpen(open) }}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>About</AlertDialogTitle>
            <AlertDialogDescription>
            <div className='space-y-2'>
              <span className='text-lg font-bold'>TUYU</span>
              <span className='text-sm text-muted-foreground'>0.1.0</span>
              <div className='mt-2'>
              <span className='text-sm font-semibold'>Dependencies:</span>
              <ul className='list-disc list-inside text-sm text-muted-foreground'>
                <li>aapt2 by Google</li>
                <li>zipalign by Google</li>
                <li>apksigner by Google</li>
                <li>apkeditor by REAndroid</li>
                <li>apktool by iBotPeaches</li>
                <li>scrcpy by Genymobile</li>
              </ul>
              </div>
            </div>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogAction>Close</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
      <AlertDialog open={devicesOpen} onOpenChange={(open) => { setDevicesOpen(open) }}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Devices</AlertDialogTitle>
            <AlertDialogDescription>
            {/* <div className='space-y-2'>
              <span className='text-lg font-bold'>TUYU</span>
              <span className='text-sm text-muted-foreground'>0.1.0</span>
              <div className='mt-2'>
              <span className='text-sm font-semibold'>Dependencies:</span>
              <ul className='list-disc list-inside text-sm text-muted-foreground'>
                <li>aapt2 by Google</li>
                <li>zipalign by Google</li>
                <li>apksigner by Google</li>
                <li>apkeditor by REAndroid</li>
                <li>apktool by iBotPeaches</li>
              </ul>
              </div>
            </div> */}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogAction>Close</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </main>
  );
}

export default App;