/**
 * 核心应用模块导出
 */

// Components
export { AppLayout } from './components/AppLayout';
export { Sidebar } from './components/Sidebar';
export { Header } from './components/Header';
export { MainContent } from './components/MainContent';
export { FloatingToolbar } from './components/FloatingToolbar';
export { FloatingDrawer } from './components/FloatingDrawer';
export { WordManagementDrawer } from './components/WordManagementDrawer';
export { SearchDrawer } from './components/SearchDrawer';
export { SettingsDrawer } from './components/SettingsDrawer';

// Context
export { AppProvider, useApp } from './context/AppContext';

// Routes
export * from './routes/routeConfig';

