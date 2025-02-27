import { getCurrentWindow } from '@tauri-apps/api/window';
import "./App.css";
import { Button } from '@/components/ui/button';
import { Settings, Smartphone, X } from 'lucide-react';
import { Separator } from '@/components/ui/separator';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { FaAndroid } from "react-icons/fa";

const appWindow = getCurrentWindow();

function App() {
  return (
    <main className='p-1 flex flex-col h-screen'>
      <div className="w-full flex justify-between h-10" onMouseDown={(e) => {
        if (e.buttons === 1) {
          e.detail === 2
            ? appWindow.toggleMaximize()
            : appWindow.startDragging();
        }
      }}>
        <div className='flex items-center space-x-1'>
          <h1 className='font-mono'>TUYU</h1>
          <Button variant="ghost" size="sm" className='font-mono'>
            About
          </Button>
          <Button variant="ghost" size="sm" className='font-mono'>
            Help
          </Button>
        </div>
        <div>
          <Button onClick={() => appWindow.close()} size="icon" variant="ghost" className='hover:bg-red-500'>
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
      <Separator className='my-1'/>
      <div className='h-full flex-1 flex items-center justify-center'>
        <div className='flex flex-col items-center'>
          <FaAndroid size={100} />
          <p className='text-muted-foreground'>Drag and drop folder, apk or xapk here</p>
        </div>
      </div>
    </main>
  );
}

export default App;
