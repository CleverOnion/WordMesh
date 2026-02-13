/**
 * 单词管理页面
 */

'use client';

import { useState, useEffect } from 'react';
import { WordList, WordForm, WordSearch, useWordList, useWord } from '@/modules/word';
import { SenseList, SenseForm, useSense } from '@/modules/sense';
import type { UserSense } from '@/modules/sense';
import { NoteCard } from '@/modules/note';
import { AssociationForm, useAssociation } from '@/modules/association';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Plus, BookOpen, Link2 } from 'lucide-react';
import type { UserWordAggregate } from '@/modules/word';
import type { CreateWordLinkRequest, CreateSenseWordLinkRequest } from '@/modules/association';
import { showSuccessToast, showErrorToast } from '@/shared/utils/toast';

export default function WordsPage() {
  const { words = [], isLoading, refresh, search, query, scope } = useWordList();
  const { addWord, isLoading: isAddingWord } = useWord();
  const { addSense, updateSense, deleteSense, isLoading: isSenseLoading } = useSense();
  const { createWordLink, createSenseWordLink, isLoading: isCreatingAssociation } = useAssociation();
  const [selectedWord, setSelectedWord] = useState<UserWordAggregate | null>(null);
  const [showAddWord, setShowAddWord] = useState(false);
  const [showAddSense, setShowAddSense] = useState(false);
  const [showAddWordLink, setShowAddWordLink] = useState(false);
  const [showAddSenseWordLink, setShowAddSenseWordLink] = useState(false);
  const [selectedSenseId, setSelectedSenseId] = useState<number | null>(null);
  const [editingSense, setEditingSense] = useState<UserSense | null>(null);

  // 调试：监听状态变化
  useEffect(() => {
    console.log('🔵 editingSense changed:', editingSense);
    console.log('🔵 editingSense ID:', editingSense?.id);
    console.log('🔵 editingSense text:', editingSense?.text);
  }, [editingSense]);

  useEffect(() => {
    console.log('🟢 showAddSenseWordLink changed:', showAddSenseWordLink, 'selectedSenseId:', selectedSenseId);
  }, [showAddSenseWordLink, selectedSenseId]);

  // 调试：监听所有相关状态
  useEffect(() => {
    console.log('📊 State update:', {
      editingSense: editingSense?.id,
      showAddSenseWordLink,
      selectedSenseId,
      showAddSense: showAddSense,
      selectedWord: selectedWord?.user_word.id,
    });
  }, [editingSense, showAddSenseWordLink, selectedSenseId, showAddSense, selectedWord]);

  const handleWordSelect = (word: UserWordAggregate) => {
    setSelectedWord(word);
    setShowAddSense(false);
    setShowAddWordLink(false);
    setShowAddSenseWordLink(false);
    setEditingSense(null);
  };

  const handleAddSense = async (data: import('@/modules/sense').AddSenseRequest | import('@/modules/sense').UpdateSenseRequest) => {
    if (!selectedWord?.user_word.id) return { success: false };
    const addData = data as import('@/modules/sense').AddSenseRequest;
    const result = await addSense(selectedWord.user_word.id, addData);
    if (result.success) {
      await refresh();
      setShowAddSense(false);
      showSuccessToast('义项添加成功');
    } else {
      showErrorToast(result.error || '添加义项失败');
    }
    return result;
  };

  const handleUpdateSense = async (data: import('@/modules/sense').UpdateSenseRequest) => {
    if (!editingSense?.id) return { success: false };
    const result = await updateSense(editingSense.id, data);
    if (result.success) {
      const refreshResult = await refresh();
      // 刷新后更新 selectedWord
      if (selectedWord && refreshResult?.success && refreshResult.data) {
        const updatedWord = refreshResult.data.items.find(
          (w) => w.user_word.id === selectedWord.user_word.id
        );
        if (updatedWord) {
          setSelectedWord(updatedWord);
        }
      }
      setEditingSense(null);
      showSuccessToast('义项更新成功');
    } else {
      showErrorToast(result.error || '更新义项失败');
    }
    return result;
  };

  const handleDeleteSense = async (senseId: number) => {
    console.log('handleDeleteSense called with senseId:', senseId);
    // 使用更明显的确认对话框
    const confirmed = window.confirm(
      '⚠️ 确定要删除这个义项吗？\n\n此操作不可撤销，删除后该义项及其所有关联将被永久删除。\n\n点击"确定"继续，或"取消"放弃。'
    );
    console.log('Confirm dialog result:', confirmed);
    if (!confirmed) {
      console.log('User cancelled deletion');
      showSuccessToast('已取消删除操作');
      return;
    }
    console.log('User confirmed deletion, calling deleteSense API');
    showSuccessToast('正在删除义项...');
    const result = await deleteSense(senseId);
    if (result.success) {
      console.log('Deletion successful, refreshing...');
      const refreshResult = await refresh();
      // 刷新后更新 selectedWord
      if (selectedWord && refreshResult?.success && refreshResult.data) {
        const updatedWord = refreshResult.data.items.find(
          (w) => w.user_word.id === selectedWord.user_word.id
        );
        if (updatedWord) {
          setSelectedWord(updatedWord);
        }
      }
      showSuccessToast('✅ 义项删除成功');
    } else {
      console.error('Deletion failed:', result.error);
      showErrorToast(result.error || '删除义项失败');
    }
  };

  const handleTogglePrimary = async (senseId: number, isPrimary: boolean) => {
    console.log('handleTogglePrimary called with senseId:', senseId, 'isPrimary:', isPrimary);
    const result = await updateSense(senseId, { is_primary: isPrimary });
    if (result.success) {
      const refreshResult = await refresh();
      // 刷新后更新 selectedWord
      if (selectedWord && refreshResult?.success && refreshResult.data) {
        const updatedWord = refreshResult.data.items.find(
          (w) => w.user_word.id === selectedWord.user_word.id
        );
        if (updatedWord) {
          setSelectedWord(updatedWord);
        }
      }
      showSuccessToast(isPrimary ? '已设为主义项' : '已取消主义项');
    } else {
      showErrorToast(result.error || '操作失败');
    }
  };

  const handleCreateWordLink = async (data: CreateWordLinkRequest | CreateSenseWordLinkRequest) => {
    if (!('word_a_id' in data)) {
      return { success: false };
    }
    const wordLinkData = data as CreateWordLinkRequest;
    const result = await createWordLink(wordLinkData);
    if (result.success) {
      await refresh();
      setShowAddWordLink(false);
      showSuccessToast('词-词关联创建成功');
      return { success: true };
    } else {
      showErrorToast(result.error || '创建关联失败');
      return { success: false };
    }
  };

  const handleCreateSenseWordLink = async (data: CreateWordLinkRequest | CreateSenseWordLinkRequest) => {
    if ('sense_id' in data) {
      const senseWordLinkData = data as CreateSenseWordLinkRequest;
      const result = await createSenseWordLink(senseWordLinkData);
      if (result.success) {
        await refresh();
        setShowAddSenseWordLink(false);
        setSelectedSenseId(null);
        showSuccessToast('义-词关联创建成功');
        return { success: true };
      } else {
        showErrorToast(result.error || '创建关联失败');
        return { success: false };
      }
    }
    return { success: false };
  };

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold flex items-center gap-2">
            <BookOpen className="size-8" />
            我的词网
          </h1>
          <p className="text-muted-foreground mt-2">
            管理和组织您的单词知识网络
          </p>
        </div>
        <Button onClick={() => setShowAddWord(true)}>
          <Plus className="size-4 mr-2" />
          添加单词
        </Button>
      </div>

      {/* 搜索栏 */}
      <Card>
        <CardContent className="pt-6">
          <WordSearch query={query || ''} scope={scope || 'both'} onSearch={search} />
        </CardContent>
      </Card>

      <div className="grid gap-6 lg:grid-cols-3">
        {/* 单词列表 */}
        <div className="lg:col-span-1">
          <Card>
            <CardHeader>
              <CardTitle>单词列表</CardTitle>
              <CardDescription>
                {words?.length || 0} 个单词
              </CardDescription>
            </CardHeader>
            <CardContent>
              {isLoading ? (
                <p className="text-muted-foreground text-center py-8">加载中...</p>
              ) : (
                <WordList
                  words={words}
                  onSelect={handleWordSelect}
                  selectedWordId={selectedWord?.user_word.id || null}
                />
              )}
            </CardContent>
          </Card>
        </div>

        {/* 单词详情 */}
        <div className="lg:col-span-2 space-y-4">
          {selectedWord ? (
            <>
              {/* 单词信息 */}
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>{selectedWord.word.text}</CardTitle>
                      <CardDescription>
                        单词 ID: {selectedWord.word.id} | 用户词项 ID: {selectedWord.user_word.id}
                      </CardDescription>
                    </div>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setShowAddWordLink(true)}
                    >
                      <Link2 className="size-4 mr-2" />
                      添加词-词关联
                    </Button>
                  </div>
                </CardHeader>
                <CardContent className="space-y-4">
                  {selectedWord.user_word.tags.length > 0 && (
                    <div>
                      <p className="text-sm font-medium mb-2">标签</p>
                      <div className="flex flex-wrap gap-2">
                        {selectedWord.user_word.tags.map((tag, index) => (
                          <span
                            key={index}
                            className="px-2 py-1 bg-primary/10 text-primary rounded-md text-sm"
                          >
                            {tag}
                          </span>
                        ))}
                      </div>
                    </div>
                  )}

                  {/* 笔记 */}
                  <NoteCard
                    node={{ type: 'word', id: selectedWord.user_word.id || 0 }}
                    note={selectedWord.user_word.note}
                    title="单词笔记"
                    onUpdate={refresh}
                  />
                </CardContent>
              </Card>

              {/* 义项列表 */}
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>义项</CardTitle>
                      <CardDescription>
                        {selectedWord.user_word.senses.length} 个义项
                      </CardDescription>
                    </div>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setShowAddSense(true)}
                    >
                      <Plus className="size-4 mr-2" />
                      添加义项
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  <SenseList
                    senses={selectedWord.user_word.senses}
                    onEdit={(sense) => {
                      console.log('🟡 Edit button clicked, sense:', sense);
                      console.log('🟡 Setting editingSense to:', sense);
                      // 立即显示toast确认按钮被点击
                      showSuccessToast('编辑按钮已点击！正在打开编辑表单...');
                      // 使用函数式更新确保状态正确更新
                      setEditingSense(() => {
                        console.log('🟡 setEditingSense callback called with:', sense);
                        return sense;
                      });
                      setShowAddSense(false);
                      setShowAddSenseWordLink(false);
                      console.log('🟡 State update called, editingSense should be:', sense);
                    }}
                    onDelete={handleDeleteSense}
                    onTogglePrimary={handleTogglePrimary}
                    onAddAssociation={(senseId) => {
                      console.log('🟢 Add association button clicked, senseId:', senseId);
                      console.log('🟢 Setting selectedSenseId to:', senseId);
                      console.log('🟢 Setting showAddSenseWordLink to: true');
                      // 立即显示toast确认按钮被点击
                      showSuccessToast('添加关联按钮已点击！正在打开关联表单...');
                      // 使用函数式更新确保状态正确更新
                      setSelectedSenseId(() => {
                        console.log('🟢 setSelectedSenseId callback called with:', senseId);
                        return senseId;
                      });
                      setShowAddSenseWordLink(() => {
                        console.log('🟢 setShowAddSenseWordLink callback called with: true');
                        return true;
                      });
                      setShowAddSense(false);
                      setEditingSense(null);
                      console.log('🟢 State updates called');
                    }}
                  />
                </CardContent>
              </Card>

              {/* 添加义项表单 */}
              {showAddSense && (
                <Card>
                  <CardHeader>
                    <CardTitle>添加新义项</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <SenseForm
                      onSubmit={handleAddSense}
                      onCancel={() => setShowAddSense(false)}
                      isLoading={isSenseLoading}
                    />
                  </CardContent>
                </Card>
              )}

              {/* 编辑义项表单 - 使用 Dialog 组件 */}
              <Dialog open={!!editingSense} onOpenChange={(open) => {
                if (!open) {
                  setEditingSense(null);
                }
              }}>
                <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
                  <DialogHeader>
                    <DialogTitle className="text-primary text-xl font-bold">
                      ✏️ 编辑义项
                    </DialogTitle>
                    <DialogDescription className="text-base">
                      编辑义项 ID: <strong>{editingSense?.id}</strong> | 义项内容: {editingSense?.text.substring(0, 50)}
                    </DialogDescription>
                  </DialogHeader>
                  <div className="pt-4">
                    <SenseForm
                      sense={editingSense!}
                      onSubmit={handleUpdateSense}
                      onCancel={() => {
                        console.log('Cancelling edit form');
                        setEditingSense(null);
                      }}
                      isLoading={isSenseLoading}
                    />
                  </div>
                </DialogContent>
              </Dialog>

              {/* 添加词-词关联表单 */}
              {showAddWordLink && (
                <Card>
                  <CardHeader>
                    <CardTitle>添加词-词关联</CardTitle>
                    <CardDescription>
                      创建两个单词之间的关联关系（形近词或词根/词缀关系）
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <AssociationForm
                      type="word"
                      onSubmit={handleCreateWordLink}
                      onCancel={() => setShowAddWordLink(false)}
                      isLoading={isCreatingAssociation}
                      wordAId={selectedWord?.word.id}
                      words={words}
                    />
                  </CardContent>
                </Card>
              )}

              {/* 添加义-词关联表单 - 使用 Dialog 组件 */}
              <Dialog open={showAddSenseWordLink} onOpenChange={(open) => {
                if (!open) {
                  setShowAddSenseWordLink(false);
                  setSelectedSenseId(null);
                }
              }}>
                <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
                  <DialogHeader>
                    <DialogTitle className="text-primary text-xl font-bold">
                      🔗 添加义-词关联
                    </DialogTitle>
                    <DialogDescription className="text-base">
                      创建义项与单词之间的语义关联（同义词、反义词或相关词）
                      {selectedSenseId && ` - 义项 ID: ${selectedSenseId}`}
                    </DialogDescription>
                  </DialogHeader>
                  <div className="pt-4">
                    <AssociationForm
                      type="sense"
                      onSubmit={handleCreateSenseWordLink}
                      onCancel={() => {
                        console.log('Cancelling sense-word link form');
                        setShowAddSenseWordLink(false);
                        setSelectedSenseId(null);
                      }}
                      isLoading={isCreatingAssociation}
                      senseId={selectedSenseId || undefined}
                      words={words}
                      selectedWord={selectedWord}
                    />
                  </div>
                </DialogContent>
              </Dialog>
            </>
          ) : (
            <Card>
              <CardContent className="py-12 text-center">
                <p className="text-muted-foreground">
                  请从左侧选择一个单词查看详情
                </p>
              </CardContent>
            </Card>
          )}
        </div>
      </div>

      {/* 添加单词表单 */}
      {showAddWord && (
        <Card>
          <CardHeader>
            <CardTitle>添加新单词</CardTitle>
          </CardHeader>
          <CardContent>
            <WordForm
              onSubmit={async (data) => {
                const result = await addWord(data);
                if (result.success) {
                  await refresh();
                  setShowAddWord(false);
                }
                return result;
              }}
              onCancel={() => setShowAddWord(false)}
              isLoading={isAddingWord}
            />
          </CardContent>
        </Card>
      )}
    </div>
  );
}

