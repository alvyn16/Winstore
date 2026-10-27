import { ChevronLeft, ChevronRight, Laptop, Smartphone, Tablet, Code2, Gamepad2, Music, Wrench, Globe, MessagesSquare, FileText, Palette } from 'lucide-react';
import type { AppItem } from '../types';
import { AppCard } from './AppCard';

export const categories = [
  { name: 'Browsers', icon: Globe, style: 'browsers' },
  { name: 'Communications', icon: MessagesSquare, style: 'communications' },
  { name: 'Development', icon: Code2, style: 'development' },
  { name: 'Documents', icon: FileText, style: 'office' },
  { name: 'Gaming', icon: Gamepad2, style: 'gaming' },
  { name: 'Audio & Video', icon: Music, style: 'audio' },
  { name: 'Graphics', icon: Palette, style: 'graphics' },
  { name: 'Utilities', icon: Wrench, style: 'utilities' },
];

export function Showcase({ apps, slide, setSlide, onOpen, onCategory }: {
  apps: AppItem[]; slide: number; setSlide: (slide: number) => void;
  onOpen: (app: AppItem) => void; onCategory: (category: string) => void;
}) {
  const gaming = slide === 1;
  const development = slide === 2;
  const title = gaming ? 'Gaming' : development ? 'Make something great' : 'On the Go';
  const category = gaming ? 'Gaming' : development ? 'Development' : 'Documents';
  const everyday = ['Mozilla.Firefox', 'Discord.Discord', 'Mozilla.Thunderbird', 'Joplin.Joplin', 'Obsidian.Obsidian', '7zip.7zip', 'RARLab.WinRAR', '9NKSQGP7F2NH', 'Brave.Brave'];
  const collection = everyday.flatMap(id => { const app = apps.find(app => app.id === id); return app ? [app] : []; });
  return <section className={`showcase ${gaming ? 'showcase-gaming' : development ? 'showcase-development' : ''}`} aria-label="Featured collections">
    <div className="showcase-intro">
      {gaming ? <Gamepad2 className="collection-symbol" /> : development ? <Code2 className="collection-symbol" /> :
        <div className="device-illustration" aria-hidden="true"><Smartphone /><Tablet /><Laptop /><Smartphone /></div>}
      <h1>{title}</h1>
      <p>{gaming ? 'A world of games. Ready for your PC.' : development ? 'Your next idea starts with the right tools.' : 'Great apps for wherever life takes you'}</p>
      <button className="collection-button" onClick={() => onCategory(category)}>{gaming ? 'Explore Gaming' : development ? 'Discover Developer Tools' : 'Explore Everyday Apps'}</button>
    </div>
    {!gaming && !development && <div className="showcase-apps">{collection.map(app => <AppCard key={app.id} app={app} onOpen={onOpen} />)}</div>}
    <div className="carousel-controls">
      <button aria-label="Previous collection" onClick={() => setSlide((slide + 2) % 3)}><ChevronLeft size={17} /></button>
      <div className="carousel-dots">{[0, 1, 2].map(index => <button key={index} aria-label={`Show collection ${index + 1}`} aria-current={slide === index ? 'true' : undefined} className={slide === index ? 'active' : ''} onClick={() => setSlide(index)} />)}</div>
      <button aria-label="Next collection" onClick={() => setSlide((slide + 1) % 3)}><ChevronRight size={17} /></button>
    </div>
  </section>;
}

export function FeaturedTiles({ apps, onOpen }: { apps: AppItem[]; onOpen: (app: AppItem) => void }) {
  const names = ['Mozilla.Firefox', 'Spotify.Spotify', 'Microsoft.PowerToys', 'Microsoft.VisualStudioCode', 'VideoLAN.VLC', 'Discord.Discord'];
  return <section className="featured-tiles" aria-label="Handpicked apps">{names.map((id, index) => {
    const app = apps.find(item => item.id === id);
    return app ? <AppCard key={id} app={app} onOpen={onOpen} className={`feature-tile tile-${index}`} /> : null;
  })}</section>;
}
