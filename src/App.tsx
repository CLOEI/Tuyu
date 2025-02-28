import { getCurrentWindow, PhysicalPosition } from '@tauri-apps/api/window';
import "./App.css";
import { Button } from '@/components/ui/button';
import { Box, Settings, Smartphone, X } from 'lucide-react';
import { Separator } from '@/components/ui/separator';
import {
  Select,
  SelectContent,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { FaAndroid, FaJava } from "react-icons/fa";
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog"
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';

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

function App() {
  const [appDetail, setAppDetail] = useState<AppDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [javaPath, setJavaPath] = useState<string | null>(null);
  
  useEffect(() => {
    let unlistenDragAndDrop: UnlistenFn;

    (async () => {
      unlistenDragAndDrop = await listen<{ paths: string[], position: PhysicalPosition }>("tauri://drag-drop", (e) => {
        invoke<AppDetail>("get_apk_detail", { path: e.payload.paths[0] }).then((data) => {
          setAppDetail(data);
        })
      })
    })();

    invoke<string|null>("get_java").then((path) => {
      if (path) {
        setLoading(false);
        setJavaPath(path);
      }
    })

    return () => {
      if (unlistenDragAndDrop) {
        unlistenDragAndDrop();
      }
    }
  }, []);

  return (
    <main className='p-1 flex flex-col h-screen'>
      <div className="w-full flex justify-between h-10" onMouseDown={(e) => {
        if (e.buttons === 1 && !(e.target instanceof HTMLButtonElement)) {
          e.detail === 2
            ? appWindow.toggleMaximize()
            : appWindow.startDragging();
        }
      }}>
        <div className='flex items-center space-x-1'>
          <h1 className='font-mono'>TUYU</h1>
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button variant="ghost" size="sm" className='font-mono'>
                About
              </Button>
            </AlertDialogTrigger>
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
      <div className='flex items-center justify-between h-10'>
        <div className='flex items-center space-x-1'>
          <Select disabled>
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder="Device not connected" />
            </SelectTrigger>
            <SelectContent></SelectContent>
          </Select>
          <Button size="icon" variant="ghost">
            <Smartphone />
          </Button>
          <Separator orientation='vertical' className='h-full'/>
          <Button variant="ghost" size="sm" className='font-mono'>
            Application
          </Button>
        </div>
        <div>
          <Button size="icon" variant="ghost">
            <Settings />
          </Button>
        </div>
      </div>
      <Separator className='mt-1'/>
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
          <div className='flex w-full h-full'>
            <div className='flex-1'>
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
                <Button onClick={() => { console.log("Decompiling...") }}>
                  Decompile
                </Button>
                <Button onClick={() => { console.log("Compiling...") }}>
                  Compile
                </Button>
                <Button onClick={() => { console.log("Signing...") }}>
                  Sign
                </Button>
                <Button onClick={() => { console.log("Converting xapk to apk...") }}>
                  Convert xapk to apk
                </Button>
                <Button onClick={() => { console.log("Installing...") }}>
                  Install
                </Button>
              </div>
            </div>
            <div className='flex-1 border-border border-l-2 p-2 ml-4'>
              <div className='overflow-y-auto'>
              </div>
            </div>
          </div>
        )
      }
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
    </main>
  );
}

export default App;