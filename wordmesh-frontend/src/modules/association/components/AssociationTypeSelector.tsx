/**
 * 关联类型选择器组件
 */

'use client';

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import type { WordLinkKind, SenseWordLinkKind } from '../types/association.types';
import {
  WORD_LINK_KIND_LABELS,
  SENSE_WORD_LINK_KIND_LABELS,
} from '../types/association.types';

interface AssociationTypeSelectorProps {
  type: 'word' | 'sense';
  value?: WordLinkKind | SenseWordLinkKind;
  onValueChange: (value: WordLinkKind | SenseWordLinkKind) => void;
  className?: string;
}

export function AssociationTypeSelector({
  type,
  value,
  onValueChange,
  className,
}: AssociationTypeSelectorProps) {
  if (type === 'word') {
    const wordKinds: WordLinkKind[] = ['similar_form', 'root_affix'];
    return (
      <Select
        value={value}
        onValueChange={(val) => onValueChange(val as WordLinkKind)}
      >
        <SelectTrigger className={className}>
          <SelectValue placeholder="选择关联类型" />
        </SelectTrigger>
        <SelectContent>
          {wordKinds.map((kind) => (
            <SelectItem key={kind} value={kind}>
              {WORD_LINK_KIND_LABELS[kind]}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    );
  }

  const senseKinds: SenseWordLinkKind[] = ['synonym', 'antonym', 'related'];
  return (
    <Select
      value={value}
      onValueChange={(val) => onValueChange(val as SenseWordLinkKind)}
    >
      <SelectTrigger className={className}>
        <SelectValue placeholder="选择关联类型" />
      </SelectTrigger>
      <SelectContent>
        {senseKinds.map((kind) => (
          <SelectItem key={kind} value={kind}>
            {SENSE_WORD_LINK_KIND_LABELS[kind]}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

