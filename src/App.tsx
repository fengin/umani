import { Routes, Route } from 'react-router-dom';
import Sidebar from './components/Sidebar';
import WorkspacePage from './pages/Workspace';
import EditorPage from './pages/Editor';
import SkillsPage from './pages/Skills';
import SettingsPage from './pages/Settings';

function App() {
  return (
    <div className="app-layout">
      <Sidebar />
      <main className="content">
        <Routes>
          <Route path="/" element={<WorkspacePage />} />
          <Route path="/editor" element={<EditorPage />} />
          <Route path="/skills" element={<SkillsPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Routes>
      </main>
    </div>
  );
}

export default App;
