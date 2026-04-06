function stemFromUrl() {
    var m = location.pathname.match(/^\/(note|f)\/(.+)/);
    return m ? decodeURIComponent(m[2]) : null;
}

function syncView(scroll) {
    var stem = stemFromUrl();
    document.body.toggleAttribute('data-note', !!stem);
    document.querySelectorAll('.note-item.active-note').forEach(function(el) {
        el.classList.remove('active-note');
    });
    if (stem) {
        var el = document.querySelector('.note-item[data-stem="' + CSS.escape(stem) + '"]');
        if (el) {
            el.classList.add('active-note');
            if (scroll) el.scrollIntoView({block: 'nearest'});
        }
    }
    var h2 = document.querySelector('#note-content h2');
    document.title = h2 ? h2.textContent + ' \u{2013} weave' : 'weave';
}

function showNote(e) {
    if (e && e.currentTarget) {
        document.querySelectorAll('.note-item.active-note').forEach(function(el) {
            el.classList.remove('active-note');
        });
        e.currentTarget.classList.add('active-note');
        e.currentTarget.style.position = 'relative';
        document.body.dataset.note = '1';
    }
}

function showList() {
    delete document.body.dataset.note;
}

function goBack() {
    if (history.length > 1) history.back();
    else location.href = '/';
}

function toggleFocus() {
    if (window.innerWidth < 768) return;
    var prose = document.querySelector('#note-content .prose');
    var titleBar = document.querySelector('#note-content > div:first-child > div');
    if (document.body.classList.contains('focus-mode')) {
        var els = [prose, titleBar].filter(Boolean);
        els.forEach(function(el) {
            el.style.maxWidth = el.offsetWidth + 'px';
            el.style.marginLeft = getComputedStyle(el).marginLeft;
            el.style.marginRight = getComputedStyle(el).marginRight;
        });
        document.body.classList.add('focus-expanding');
        document.body.classList.remove('focus-mode');
        requestAnimationFrame(function() {
            els.forEach(function(el) {
                el.style.marginLeft = '';
                el.style.marginRight = '';
            });
        });
        document.getElementById('sidebar').addEventListener('transitionend', function handler(e) {
            if (e.propertyName === 'width') {
                els.forEach(function(el) { el.style.maxWidth = ''; });
                document.body.classList.remove('focus-expanding');
                document.getElementById('sidebar').removeEventListener('transitionend', handler);
            }
        });
    } else {
        var proseWidth = prose ? prose.offsetWidth : 0;
        if (prose) {
            document.body.style.setProperty('--focus-prose-width', proseWidth + 'px');
        }

        // Calculate final centered margin for the title bar so it
        // transitions in sync with the sidebar collapse (both use the
        // same 0.3s ease timing), avoiding the non-linear motion that
        // margin:auto causes when the container width is also changing.
        if (titleBar) {
            var sidebarWidth = sidebar.offsetWidth;
            var targetWidth = proseWidth || titleBar.offsetWidth;
            var finalMargin = Math.max(0, (titleBar.offsetWidth + sidebarWidth - targetWidth) / 2);

            titleBar.style.maxWidth = targetWidth + 'px';
            titleBar.style.marginLeft = '0px';
            titleBar.style.marginRight = '0px';
        }

        document.body.classList.add('focus-mode');

        if (titleBar) {
            requestAnimationFrame(function() {
                titleBar.style.marginLeft = finalMargin + 'px';
                titleBar.style.marginRight = finalMargin + 'px';
            });
            sidebar.addEventListener('transitionend', function handler(e) {
                if (e.propertyName === 'width') {
                    titleBar.style.marginLeft = '';
                    titleBar.style.marginRight = '';
                    titleBar.style.maxWidth = '';
                    sidebar.removeEventListener('transitionend', handler);
                }
            });
        }
    }
}

function activateSearch() {
    document.getElementById('sidebar-header').classList.add('search-active');
    document.getElementById('filter-clear').classList.remove('hidden');
    document.getElementById('filter-input').focus();
}

function activateClip() {
    var tb = document.getElementById('title-bar');
    if (tb) {
        tb.classList.add('clip-active');
        var ci = document.getElementById('clip-input');
        if (ci) ci.focus();
    }
}

function deactivateClip() {
    var tb = document.getElementById('title-bar');
    if (tb) tb.classList.remove('clip-active');
    var ci = document.getElementById('clip-input');
    if (ci) {
        ci.value = '';
        ci.blur();
    }
}

function deactivateSearch() {
    document.getElementById('sidebar-header').classList.remove('search-active');
    var fi = document.getElementById('filter-input');
    fi.value = '';
    fi.blur();
    htmx.ajax('POST', '/f/search', {target: '#search-list', values: {query: ''}});
    var searchRow = document.getElementById('search-row');
    searchRow.addEventListener('transitionend', function handler() {
        document.getElementById('filter-clear').classList.add('hidden');
        searchRow.removeEventListener('transitionend', handler);
    });
}

document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape') {
        if (document.body.classList.contains('focus-mode')) {
            toggleFocus();
            return;
        }
        if (document.activeElement === document.getElementById('clip-input')) {
            deactivateClip();
            return;
        }
        if (document.activeElement === document.getElementById('filter-input')) {
            deactivateSearch();
            return;
        }
    }
    if (e.key === 'Tab' && document.activeElement === document.getElementById('filter-input')) {
        e.preventDefault();
        deactivateSearch();
        var first = document.querySelector('#search-list .note-item');
        if (first) {
            first.click();
            first.scrollIntoView({block: 'nearest'});
        }
        return;
    }
    var t = document.activeElement.tagName;
    if (t === 'INPUT' || t === 'TEXTAREA' || t === 'SELECT') return;
    if (e.key === 'e') {
        var btn = document.querySelector('[aria-label="Edit note"]');
        if (btn) btn.click();
    } else if (e.key === 'f') {
        e.preventDefault();
        toggleFocus();
    } else if (e.key === 'c') {
        var clipToggle = document.getElementById('clip-toggle');
        if (clipToggle) {
            e.preventDefault();
            activateClip();
        }
    } else if (e.key === 's') {
        e.preventDefault();
        activateSearch();
    } else if (e.key === 'j' || e.key === 'k') {
        var items = Array.from(document.querySelectorAll('#search-list .note-item'));
        if (!items.length) return;
        var active = document.querySelector('.note-item.active-note');
        var idx = active ? items.indexOf(active) : -1;
        var next = e.key === 'j' ? idx + 1 : idx - 1;
        if (next < 0 || next >= items.length) return;
        items[next].click();
        items[next].scrollIntoView({block: 'nearest'});
    }
});

document.addEventListener('DOMContentLoaded', function() {
    syncView(true);
});

// Use event delegation so listeners survive HTMX history cache restores
// which replace DOM elements and lose directly-bound listeners.
document.addEventListener('input', function(e) {
    if (e.target.id === 'filter-input') {
        var inSearchMode = document.getElementById('sidebar-header').classList.contains('search-active');
        if (!inSearchMode) {
            document.getElementById('filter-clear').classList.toggle('hidden', !e.target.value);
        }
    }
});

document.addEventListener('click', function(e) {
    if (e.target.closest('#clip-toggle')) {
        activateClip();
    } else if (e.target.closest('#clip-cancel')) {
        deactivateClip();
    } else if (e.target.closest('#search-toggle')) {
        activateSearch();
    } else if (e.target.closest('#filter-clear')) {
        deactivateSearch();
    }
});

window.addEventListener('popstate', function() {
    var stem = stemFromUrl();
    if (stem) {
        htmx.ajax('GET', '/f/' + encodeURIComponent(stem), {target: '#note-content'});
    }
    syncView(true);
});

document.addEventListener('htmx:historyRestore', function() {
    syncView(true);
    var filterInput = document.getElementById('filter-input');
    var filterClear = document.getElementById('filter-clear');
    if (filterInput && filterClear) {
        filterClear.classList.toggle('hidden', !filterInput.value);
        htmx.ajax('POST', '/f/search', {target: '#search-list', values: {query: filterInput.value}});
        if (filterInput.value) {
            document.getElementById('sidebar-header').classList.add('search-active');
        }
    }
});

document.addEventListener('htmx:afterRequest', function(e) {
    if (e.detail.elt && e.detail.elt.id === 'clip-input') {
        deactivateClip();
    }
});

document.addEventListener('htmx:afterSettle', function(e) {
    if (e.detail.target.id === 'note-content') {
        syncView(false);
    }
});

function showNoteError(message) {
    var nc = document.getElementById('note-content');
    if (!nc) return;
    nc.innerHTML =
        '<div class="p-4 border-b border-gray-200 dark:border-gray-700 flex-shrink-0">' +
          '<div class="flex items-center gap-3">' +
            '<button class="md:hidden p-1 -ml-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700" onclick="goBack()" aria-label="Back to notes">' +
              '<svg class="w-6 h-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/></svg>' +
            '</button>' +
            '<span class="invisible font-black text-xl flex-grow">\u00a0</span>' +
          '</div>' +
        '</div>' +
        '<div class="flex-grow flex items-center justify-center">' +
          '<div class="flex flex-col items-center justify-center p-8 text-center">' +
            '<h2 class="text-xl font-bold text-gray-400 dark:text-gray-500">' + message + '</h2>' +
          '</div>' +
        '</div>';
}

document.addEventListener('htmx:sendError', function(e) {
    if (e.detail.target && e.detail.target.id === 'note-content') {
        showNoteError('note could not be loaded');
    }
});

document.addEventListener('htmx:responseError', function(e) {
    if (e.detail.target && e.detail.target.id === 'note-content') {
        showNoteError('note could not be loaded');
    }
});

(function() {
    var source = new EventSource('/events');
    source.addEventListener('notes-updated', function(e) {
        htmx.trigger(document.body, 'notes-updated');
        try {
            var data = JSON.parse(e.data);
            var current = stemFromUrl();
            if (current && current === data.stem) {
                if (data.removed) showNoteError('note was removed');
                else htmx.ajax('GET', '/f/' + encodeURIComponent(data.stem), {target: '#note-content'});
            }
        } catch (err) {}
    });
})();
