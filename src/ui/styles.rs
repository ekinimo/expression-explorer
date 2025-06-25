pub const APP_CONTAINER: &str = "min-h-screen bg-gray-50";
pub const CONTENT_CONTAINER: &str = "container mx-auto px-6 py-6 max-w-7xl";
pub const PANEL: &str = "bg-white p-6 rounded-lg shadow border border-gray-200";
pub const CARD: &str = "bg-white p-4 rounded-md shadow-sm border border-gray-200";

pub const GRID_1: &str = "grid grid-cols-1 gap-6";
pub const GRID_2: &str = "grid grid-cols-1 lg:grid-cols-2 gap-6";
pub const GRID_3: &str = "grid grid-cols-1 lg:grid-cols-3 gap-6";

pub const FLEX_ROW: &str = "flex items-center gap-4";
pub const FLEX_COL: &str = "flex flex-col gap-4";
pub const FLEX_BETWEEN: &str = "flex items-center justify-between";
pub const FLEX_CENTER: &str = "flex items-center justify-center";

pub const NAV_BAR: &str = "bg-white border-b border-gray-200 shadow-sm";
pub const NAV_CONTAINER: &str = "container mx-auto px-6";
pub const NAV_LIST: &str = "flex space-x-8";
pub const NAV_TITLE: &str = "text-xl font-bold text-gray-900 py-4";

pub const NAV_ITEM_BASE: &str =
    "py-4 px-1 border-b-2 font-medium text-sm cursor-pointer transition-all duration-200";
pub const NAV_ITEM_ACTIVE: &str =
    "py-4 px-1 border-b-2 border-blue-500 text-blue-600 font-medium text-sm cursor-pointer";
pub const NAV_ITEM_INACTIVE: &str = "py-4 px-1 border-b-2 border-transparent text-gray-600 hover:text-gray-900 hover:border-gray-300 font-medium text-sm cursor-pointer transition-all duration-200";

pub const TITLE_MAIN: &str = "text-3xl font-bold text-gray-900 mb-6";
pub const TITLE_PAGE: &str = "text-2xl font-bold text-gray-800 mb-4";
pub const TITLE_SECTION: &str = "text-xl font-semibold text-gray-800 mb-4";
pub const TITLE_SUBSECTION: &str = "text-lg font-medium text-gray-700 mb-3";

pub const TEXT_BODY: &str = "text-gray-700";
pub const TEXT_SMALL: &str = "text-sm text-gray-600";
pub const TEXT_TINY: &str = "text-xs text-gray-500";
pub const TEXT_MUTED: &str = "text-gray-500";
pub const TEXT_MONO: &str = "font-mono text-sm";
pub const TEXT_MONO_LG: &str = "font-mono text-base";

pub const LABEL: &str = "block text-sm font-medium text-gray-700 mb-2";
pub const LABEL_INLINE: &str = "text-sm font-medium text-gray-700";
pub const LABEL_REQUIRED: &str = "block text-sm font-medium text-gray-700 mb-2 after:content-['*'] after:text-red-500 after:ml-1";

pub const INPUT_BASE: &str = "w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors";
pub const INPUT: &str = "w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors";
pub const INPUT_ERROR: &str = "w-full px-3 py-2 border border-red-300 rounded-md shadow-sm focus:ring-2 focus:ring-red-500 focus:border-red-500 transition-colors";

pub const TEXTAREA: &str = "w-full h-32 px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm transition-colors resize-vertical";
pub const TEXTAREA_LG: &str = "w-full h-48 px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm transition-colors resize-vertical";
pub const TEXTAREA_CODE: &str = "w-full h-32 px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm transition-colors resize-vertical bg-gray-50";

pub const BTN_PRIMARY: &str = "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-all duration-200 font-medium shadow-sm";
pub const BTN_PRIMARY_LG: &str = "px-6 py-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-all duration-200 font-medium shadow-sm text-lg";

pub const BTN_SECONDARY: &str = "px-4 py-2 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300 focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 transition-all duration-200 font-medium shadow-sm";
pub const BTN_OUTLINE: &str = "px-4 py-2 border border-gray-300 text-gray-700 rounded-md hover:bg-gray-50 focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 transition-all duration-200 font-medium shadow-sm";

pub const BTN_SUCCESS: &str = "px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 focus:ring-2 focus:ring-green-500 focus:ring-offset-2 transition-all duration-200 font-medium shadow-sm";
pub const BTN_WARNING: &str = "px-4 py-2 bg-yellow-600 text-white rounded-md hover:bg-yellow-700 focus:ring-2 focus:ring-yellow-500 focus:ring-offset-2 transition-all duration-200 font-medium shadow-sm";
pub const BTN_DANGER: &str = "px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 focus:ring-2 focus:ring-red-500 focus:ring-offset-2 transition-all duration-200 font-medium shadow-sm";

pub const BTN_SM: &str = "px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 transition-all duration-200 font-medium";
pub const BTN_SM_OUTLINE: &str = "px-3 py-1 text-sm border border-gray-300 text-gray-700 rounded hover:bg-gray-50 focus:ring-2 focus:ring-gray-500 transition-all duration-200 font-medium";

pub const ERROR_BOX: &str = "p-4 bg-red-50 border border-red-200 text-red-800 rounded-md shadow-sm";
pub const SUCCESS_BOX: &str =
    "p-4 bg-green-50 border border-green-200 text-green-800 rounded-md shadow-sm";
pub const WARNING_BOX: &str =
    "p-4 bg-yellow-50 border border-yellow-200 text-yellow-800 rounded-md shadow-sm";
pub const INFO_BOX: &str =
    "p-4 bg-blue-50 border border-blue-200 text-blue-800 rounded-md shadow-sm";

pub const DISPLAY_BOX: &str = "p-4 bg-gray-100 border border-gray-200 rounded-md";
pub const CODE_BOX: &str =
    "p-4 bg-gray-900 text-gray-100 rounded-md font-mono text-sm overflow-x-auto";
pub const RESULT_BOX: &str = "p-4 bg-green-50 border border-green-200 rounded-md font-mono text-sm";

pub const SPACE_Y_1: &str = "space-y-1";
pub const SPACE_Y_2: &str = "space-y-2";
pub const SPACE_Y_3: &str = "space-y-3";
pub const SPACE_Y_4: &str = "space-y-4";
pub const SPACE_Y_6: &str = "space-y-6";
pub const SPACE_Y_8: &str = "space-y-8";

pub const SPACE_X_2: &str = "space-x-2";
pub const SPACE_X_4: &str = "space-x-4";
pub const SPACE_X_6: &str = "space-x-6";

pub const MB_2: &str = "mb-2";
pub const MB_4: &str = "mb-4";
pub const MB_6: &str = "mb-6";
pub const MT_2: &str = "mt-2";
pub const MT_4: &str = "mt-4";
pub const MT_6: &str = "mt-6";

pub const HIDDEN: &str = "hidden";
pub const VISIBLE: &str = "visible";

pub const BORDER: &str = "border border-gray-200";
pub const BORDER_DASHED: &str = "border border-dashed border-gray-300";

pub const SHADOW_SM: &str = "shadow-sm";
pub const SHADOW: &str = "shadow";
pub const SHADOW_LG: &str = "shadow-lg";

pub const ROUNDED: &str = "rounded";
pub const ROUNDED_MD: &str = "rounded-md";
pub const ROUNDED_LG: &str = "rounded-lg";
